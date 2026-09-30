const m5Client = require("../utils/m5Client");

function normalizeMachine(machine) {
  return {
    machine_id: String(machine.machine_id),
    id: String(machine.machine_id),
    hostname: machine.hostname,
    operating_system: machine.os,
    os: machine.os,
    platform: machine.os?.toLowerCase().includes("windows")
      ? "windows"
      : machine.os?.toLowerCase().includes("linux")
        ? "linux"
        : "unknown",
    status: machine.status?.toUpperCase() || "UNKNOWN",

    // These fields are not provided by M5 /machines yet.
    ip: null,
    agent_id: null,
    agent: null,
    arch: null,
    last_heartbeat: null,
    heartbeat: null,
    evidenceCount: 0,
    findingsCount: 0,
    processes: 0,
    connections: 0,
    activity: [],
  };
}

module.exports = {
  findAll: async () => {
    const machines = await m5Client.get("/machines");
    return machines.map(normalizeMachine);
  },

  findById: async (id) => {
    const machines = await m5Client.get("/machines");

    const machine = machines.find(
      (m) =>
        String(m.machine_id) === String(id) ||
        m.hostname?.toLowerCase() === String(id).toLowerCase()
    );

    return machine ? normalizeMachine(machine) : undefined;
  },

  create: async () => {
    throw new Error(
      "Machine registration is handled by M5. Product backend cannot create machines directly."
    );
  },
};