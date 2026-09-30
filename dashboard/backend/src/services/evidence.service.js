// Evidence Service
// Interfaces with evidence adapter and repository

const evidenceAdapter = require("../adapters/evidence.adapter");

module.exports = {
  getAll: async (investigationId) => {
    return await evidenceAdapter.getEvidenceList(investigationId);
  },

  getById: async (id) => {
    return await evidenceAdapter.getEvidenceById(id);
  },

  preserve: async (data) => {
    return await evidenceAdapter.preserveEvidence(data);
  }
};
