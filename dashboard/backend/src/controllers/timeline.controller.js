const timelineService = require("../services/timeline.service");

module.exports = {
  getAll: async (req, res, next) => {
    try {
      const { investigation_id } = req.query;
      const list = await timelineService.getAll(investigation_id);
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
      const event = await timelineService.create(req.body);
      res.status(201).json({
        success: true,
        data: event
      });
    } catch (err) {
      next(err);
    }
  }
};
