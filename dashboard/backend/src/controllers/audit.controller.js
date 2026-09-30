const auditService = require("../services/audit.service");

module.exports = {
  getAll: async (req, res, next) => {
    try {
      const { investigation_id } = req.query;
      const list = await auditService.getAll(investigation_id);
      res.json({
        success: true,
        data: list
      });
    } catch (err) {
      next(err);
    }
  },

  create: async (req, res, next) => {
    try {
      const entry = await auditService.create(req.body);
      res.status(201).json({
        success: true,
        data: entry
      });
    } catch (err) {
      next(err);
    }
  }
};
