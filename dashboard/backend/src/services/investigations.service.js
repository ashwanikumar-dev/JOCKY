const investigationsRepo = require("../repositories/investigations.repository");
const compilerAdapter = require("../adapters/compiler.adapter");
const m5Client = require("../utils/m5Client");

module.exports = {
  getAll: async () => {
    return await investigationsRepo.findAll();
  },

  getById: async (id) => {
    const investigation = await investigationsRepo.findById(id);

    if (!investigation) {
      return undefined;
    }

    if (!investigation.task_id) {
      return investigation;
    }

    try {
      const task = await m5Client.get(`/tasks/${investigation.task_id}`);

      const statusMap = {
        created: "PENDING",
        dispatched: "RUNNING",
        running: "RUNNING",
        completed: "COMPLETED",
        failed: "FAILED",
      };

      const status = statusMap[task.status] || investigation.status;

      // Keep Product Backend's stored state synchronized with M5.
      if (status !== investigation.status) {
        await investigationsRepo.update(investigation.investigation_id, {
          status,
        });
      }

      return {
        ...investigation,
        status,
        task_id: String(task.task_id),
        task_status: task.status,
        task_created_at: task.created_at,
        task_ended_at: task.ended_at,
      };
    } catch (error) {
      console.warn(
        `[INVESTIGATIONS] Failed to fetch task ${investigation.task_id}:`,
        error.message,
      );

      return investigation;
    }
  },

  create: async (data) => {
    const { script, target, title } = data;

    const investigationId =
      data.investigation_id || data.id || `inv-${Date.now()}`;

    let compiledResult = null;

    // 1. Compile through the real M3 compiler.
    if (script) {
      compiledResult = await compilerAdapter.compileScript(
        script,
        investigationId,
      );
    }

    // 2. Create the investigation in M5.
    let m5Investigation = null;

    if (script) {
      m5Investigation = await m5Client.post("/investigations", {
        script,
        compiled_ir: compiledResult.ir,
      });
    }

    const m5InvestigationId = m5Investigation?.investigation_id ?? null;

    // 3. Store Product Backend metadata.
    const newInv = await investigationsRepo.create({
      investigation_id: investigationId,
      id: investigationId,

      m5_investigation_id: m5InvestigationId,

      title:
        title || compiledResult?.ir?.investigation_name || "Endpoint Triage",

      target: target || "Windows Fleet",

      script: script || "",

      compiled_ir: compiledResult?.ir || null,

      status: "READY",
    });

    return {
      investigation: newInv,
      m5_investigation_id: m5InvestigationId,
      ir: compiledResult?.ir || null,
      ast: compiledResult?.ast || null,
    };
  },

  run: async (id) => {
    // 1. Find the Product investigation.
    const investigation = await investigationsRepo.findById(id);

    if (!investigation) {
      const error = new Error(`Investigation with ID '${id}' not found`);
      error.status = 404;
      throw error;
    }

    // 2. Make sure this investigation is linked to M5.
    const m5InvestigationId = investigation.m5_investigation_id;

    if (!m5InvestigationId) {
      const error = new Error(
        "Investigation is not linked to an M5 investigation",
      );
      error.status = 400;
      throw error;
    }

    // 3. Get registered M5 agents.
    const agents = await m5Client.get("/agents");

    if (!Array.isArray(agents) || agents.length === 0) {
      const error = new Error("No registered M5 agents available");
      error.status = 503;
      throw error;
    }

    // 4. Select the most recently active agent.
    const activeAgent = [...agents]
      .filter((agent) => agent.agent_id != null)
      .sort(
        (a, b) =>
          new Date(b.last_heartbeat || 0) - new Date(a.last_heartbeat || 0),
      )[0];

    if (!activeAgent) {
      const error = new Error("No usable M5 agent available");
      error.status = 503;
      throw error;
    }

    // 5. Create the real M5 task.
    const task = await m5Client.post("/tasks", {
      investigation_id: Number(m5InvestigationId),
      agent_id: Number(activeAgent.agent_id),
    });
    await investigationsRepo.update(investigation.investigation_id, {
      task_id: task.task_id,
      status: "RUNNING",
    });

    return {
      investigation_id: investigation.investigation_id,
      m5_investigation_id: Number(m5InvestigationId),
      agent_id: Number(activeAgent.agent_id),
      machine_id:
        activeAgent.machine_id != null ? Number(activeAgent.machine_id) : null,
      task,
    };
  },
};
