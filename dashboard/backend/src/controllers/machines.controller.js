const machinesService = require("../services/machines.service");

module.exports = {
  getAll: async (req, res, next) => {
    try {
      const machines = await machinesService.getAll();
      res.json({
        success: true,
        data: machines
      });
    } catch (err) {
      next(err);
    }
  },

  getById: async (req, res, next) => {
    try {
      const { id } = req.params;
      const machine = await machinesService.getById(id);
      if (!machine) {
        return res.status(404).json({
          success: false,
          error: { message: `Machine with identifier '${id}' not found` }
        });
      }
      res.json({
        success: true,
        data: machine
      });
    } catch (err) {
      next(err);
    }
  },

  register: async (req, res, next) => {
    try {
      const newMachine = await machinesService.register(req.body);
      res.status(201).json({
        success: true,
        data: newMachine
      });
    } catch (err) {
      next(err);
    }
  }
};
