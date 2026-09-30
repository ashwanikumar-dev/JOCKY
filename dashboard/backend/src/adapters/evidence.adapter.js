// JOCKY Evidence Vault Adapter
// Interface: Layer 5 (Forensic Intelligence) -> Layer 6 (Platform)
// Adheres to Handbook Contract #7 (Evidence Schema) and Section 22/24 (SHA-256 Integrity)

const evidenceRepo = require("../repositories/evidence.repository");

module.exports = {
  /**
   * Retrieve all preserved evidence artifacts
   * @param {string} [investigationId]
   */
  getEvidenceList: async (investigationId) => {
    return await evidenceRepo.findAll(investigationId);
  },

  /**
   * Retrieve single evidence object by evidence_id
   * @param {string} id
   */
  getEvidenceById: async (id) => {
    return await evidenceRepo.findById(id);
  },

  /**
   * Preserve new evidence package
   * @param {object} evidenceData
   */
  preserveEvidence: async (evidenceData) => {
    return await evidenceRepo.create(evidenceData);
  }
};
