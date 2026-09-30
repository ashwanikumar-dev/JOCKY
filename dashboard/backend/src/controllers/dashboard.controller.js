const dashboardService = require("../services/dashboard.service");

module.exports = {
  getStats: async (req, res, next) => {
    try {
      const stats = await dashboardService.getDashboardStats();
      res.json({
        success: true,
        data: stats
      });
    } catch (err) {
      next(err);
    }
  }
};
