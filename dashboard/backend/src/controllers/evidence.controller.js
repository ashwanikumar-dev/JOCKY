const evidenceService = require("../services/evidence.service");

module.exports = {
  getAll: async (req, res, next) => {
    try {
      const { investigation_id } = req.query;
      const list = await evidenceService.getAll(investigation_id);
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
      const item = await evidenceService.getById(id);
      if (!item) {
        return res.status(404).json({
          success: false,
          error: { message: `Evidence artifact '${id}' not found` }
        });
      }
      res.json({
        success: true,
        data: item
      });
    } catch (err) {
      next(err);
    }
  },

  preserve: async (req, res, next) => {
    try {
      const record = await evidenceService.preserve(req.body);
      res.status(201).json({
        success: true,
        data: record
      });
    } catch (err) {
      next(err);
    }
  }
};
