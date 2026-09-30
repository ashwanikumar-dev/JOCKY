const investigationsService = require("../services/investigations.service");

module.exports = {
  getAll: async (req, res, next) => {
    try {
      const list = await investigationsService.getAll();
      res.json({
        success: true,
        data: list,
      });
    } catch (err) {
      next(err);
    }
  },

  getById: async (req, res, next) => {
    try {
      const { id } = req.params;
      const inv = await investigationsService.getById(id);
      if (!inv) {
        return res.status(404).json({
          success: false,
          error: { message: `Investigation with ID '${id}' not found` },
        });
      }
      res.json({
        success: true,
        data: inv,
      });
    } catch (err) {
      next(err);
    }
  },

  create: async (req, res, next) => {
    try {
      const result = await investigationsService.create(req.body);

      res.status(201).json({
        success: true,
        data: result.investigation,
        ir: result.ir,
        ast: result.ast,
      });
    } catch (err) {
      next(err);
    }
  },

  run: async (req, res, next) => {
    try {
      const { id } = req.params;

      const result = await investigationsService.run(id);

      res.status(201).json({
        success: true,
        data: result,
      });
    } catch (err) {
      next(err);
    }
  },
};
