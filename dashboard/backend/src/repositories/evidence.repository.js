const m5Client = require("../utils/m5Client");

function normalizeEvidence(evidence) {
  const collectedAt = evidence.collected_at
    ? new Date(evidence.collected_at)
    : null;

  return {
    evidence_id: String(evidence.evidence_id),
    id: String(evidence.evidence_id),

    task_id: evidence.task_id != null ? String(evidence.task_id) : null,

    machine_id:
      evidence.machine_id != null ? String(evidence.machine_id) : null,

    machine: evidence.machine_id != null ? String(evidence.machine_id) : null,

    agent_id: evidence.agent_id != null ? String(evidence.agent_id) : null,

    evidence_type: evidence.category,
    type: evidence.category,

    collector: null,
    collector_version: evidence.collector_version || null,

    collection_time: evidence.collected_at || null,
    collectedAt: collectedAt
      ? collectedAt.toLocaleString("en-GB", {
          day: "2-digit",
          month: "short",
          year: "numeric",
          hour: "2-digit",
          minute: "2-digit",
          hour12: false,
          timeZone: "UTC",
        }) + " UTC"
      : null,

    sha256: evidence.sha256 || null,

    raw_data: evidence.raw || null,
    normalized_data: evidence.normalized || null,

    status:
      evidence.status === "verified"
        ? "Verified"
        : evidence.status === "integrity_failed"
          ? "Integrity Failed"
          : evidence.status
            ? String(evidence.status)
            : "Unknown",
  };
}

module.exports = {
  findAll: async (investigationId) => {
    const endpoint = investigationId
      ? `/evidence?investigation_id=${encodeURIComponent(investigationId)}`
      : "/evidence";

    const evidence = await m5Client.get(endpoint);

    return Array.isArray(evidence) ? evidence.map(normalizeEvidence) : [];
  },

  findById: async (id) => {
    const evidence = await m5Client.get(`/evidence/${encodeURIComponent(id)}`);

    return evidence ? normalizeEvidence(evidence) : undefined;
  },

  create: async () => {
    throw new Error(
      "Evidence collection is handled by M4/M5. Product backend cannot create evidence directly.",
    );
  },
};
