// Findings Service
// Interfaces with detection adapter and findings repository

const detectionAdapter = require("../adapters/detection.adapter");

module.exports = {
  getAll: async (investigationId) => {
    return await detectionAdapter.getFindingsList(investigationId);
  },

  getById: async (id) => {
    return await detectionAdapter.getFindingById(id);
  },

  create: async (data) => {
    return await detectionAdapter.recordFinding(data);
  }
};
