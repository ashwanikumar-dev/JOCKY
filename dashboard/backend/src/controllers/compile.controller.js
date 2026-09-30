const compileService = require("../services/compile.service");

module.exports = {
  compile: async (req, res, next) => {
    try {
      const { script } = req.body;
      const result = await compileService.compile(script);
      res.json({
        success: true,
        ir: result.ir,
        ast: result.ast,
        valid: result.valid
      });
    } catch (err) {
      next(err);
    }
  }
};
