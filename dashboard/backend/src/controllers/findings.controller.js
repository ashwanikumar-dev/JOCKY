const findingsService = require("../services/findings.service");

module.exports = {
  getAll: async (req, res, next) => {
    try {
      const { investigation_id } = req.query;
      const list = await findingsService.getAll(investigation_id);
      res.json({
        success: true,
        data: list
      });
    } catch (err) {
      next(err);
    }
  },

  getById: async (req, res, next) => {
    try {
      const { id } = req.params;
      const finding = await findingsService.getById(id);
      if (!finding) {
        return res.status(404).json({
          success: false,
          error: { message: `Finding '${id}' not found` }
        });
      }
      res.json({
        success: true,
        data: finding
      });
    } catch (err) {
      next(err);
    }
  },

  create: async (req, res, next) => {
    try {
      const record = await findingsService.create(req.body);
      res.status(201).json({
        success: true,
        data: record
      });
    } catch (err) {
      next(err);
    }
  }
};
