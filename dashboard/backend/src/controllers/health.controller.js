// Health Controller
module.exports = {
  checkHealth: (req, res) => {
    res.json({
      success: true,
      status: "ok",
      timestamp: new Date().toISOString(),
      uptime: process.uptime(),
      version: "v1.0.0",
      service: "JOCKY Dashboard Backend (SIH PS 26148)"
    });
  }
};
