// Dashboard Service
// Aggregates real telemetry from Product Backend repositories/adapters.

const agentAdapter = require("../adapters/agent.adapter");
const evidenceAdapter = require("../adapters/evidence.adapter");
const detectionAdapter = require("../adapters/detection.adapter");
const investigationsRepo = require("../repositories/investigations.repository");

module.exports = {
  getDashboardStats: async () => {
    // Investigations are Product Backend-owned.
    const investigations = await investigationsRepo.findAll();

    // Real M5-backed telemetry.
    const [machines, evidence] = await Promise.all([
      agentAdapter.getRegisteredAgents(),
      evidenceAdapter.getEvidenceList(),
    ]);

    // M5 findings require an investigation_id.
    // Use the latest Product Backend investigation when available.
    const activeInvestigation = investigations[0] || null;

    let findings = [];

    if (activeInvestigation?.m5_investigation_id) {
      findings = await detectionAdapter.getFindingsList(
        activeInvestigation.m5_investigation_id,
      );
    }

    const onlineNodes = machines.filter(
      (machine) => String(machine.status || "").toUpperCase() === "ONLINE",
    ).length;

    const windowsNodes = machines.filter((machine) =>
      String(machine.os || machine.operating_system || "")
        .toLowerCase()
        .includes("windows"),
    ).length;

    const linuxNodes = machines.filter((machine) =>
      String(machine.os || machine.operating_system || "")
        .toLowerCase()
        .includes("linux"),
    ).length;

    const highFindings = findings.filter(
      (finding) => String(finding.severity || "").toUpperCase() === "HIGH",
    ).length;

    const medFindings = findings.filter(
      (finding) => String(finding.severity || "").toUpperCase() === "MEDIUM",
    ).length;

    const critFindings = findings.filter(
      (finding) => String(finding.severity || "").toUpperCase() === "CRITICAL",
    ).length;

    const totalMachines = machines.length;

    const activeTitle = activeInvestigation?.title || "No Active Investigation";

    const activeStatus = activeInvestigation?.status || "IDLE";

    const grammarVersion = activeInvestigation?.grammar_version || "v0.1";

    return {
      systemStatus: {
        status: "ONLINE",
        label: "SYSTEM ONLINE",
        telemetryActive: true,

        nodeCount: totalMachines,
        nodesOnline: onlineNodes,

        investigator: {
          name: activeInvestigation?.investigator || "system",
          role: "Investigator",
        },

        activeCase: activeInvestigation
          ? `${activeTitle} · ${onlineNodes}/${totalMachines} Nodes Online`
          : "No active investigation",
      },

      kpi: {
        activeInvestigation: {
          title: "Active Investigation",
          value: activeTitle,
          sub: activeInvestigation
            ? `DSL ${grammarVersion} · ${activeStatus}`
            : "No investigation created",
        },

        enrolledEndpoints: {
          title: "Enrolled Endpoints",
          value: `${onlineNodes} / ${totalMachines} Online`,
          healthBadge:
            totalMachines > 0
              ? `${Math.round((onlineNodes / totalMachines) * 100)}% HEALTH`
              : "NO NODES",

          sub: `${windowsNodes} Windows · ${linuxNodes} Linux`,
        },

        preservedEvidence: {
          title: "Preserved Evidence",
          value: `${evidence.length} Artifacts`,
          hashBadge: "SHA-256",
          sub: "Tamper-evident chain",
        },

        securityFindings: {
          title: "Security Findings",
          value: `${findings.length} Detections`,
          highCount: highFindings,
          sub: `${medFindings} Medium · ${critFindings} Critical`,
        },
      },

      recentInvestigations: investigations,

      recentFindings: findings,
    };
  },
};
