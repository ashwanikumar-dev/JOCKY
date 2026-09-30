// JOCKY Detection Engine Adapter
// Interface: Layer 5 (Forensic Intelligence) -> Layer 6 (Platform)
// Adheres to Handbook Contract #9 and Section 26 (Rule-based Detection Engine)

const findingsRepo = require("../repositories/findings.repository");

module.exports = {
  /**
   * Retrieve triggered findings
   * @param {string} [investigationId]
   */
  getFindingsList: async (investigationId) => {
    return await findingsRepo.findAll(investigationId);
  },

  /**
   * Retrieve single finding by id
   * @param {string} id
   */
  getFindingById: async (id) => {
    return await findingsRepo.findById(id);
  },

  /**
   * Register a new finding
   * @param {object} findingData
   */
  recordFinding: async (findingData) => {
    return await findingsRepo.create(findingData);
  }
};
