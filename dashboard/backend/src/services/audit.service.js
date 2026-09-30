// Audit / Custody Service
const auditRepo = require("../repositories/audit.repository");

module.exports = {
  getAll: async (investigationId) => {
    return await auditRepo.findAll(investigationId);
  },

  create: async (data) => {
    return await auditRepo.create(data);
  }
};
