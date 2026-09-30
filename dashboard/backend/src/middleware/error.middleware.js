const logger = require("../utils/logger");

module.exports = (err, req, res, next) => {
  logger.error(`[API Error] ${req.method} ${req.originalUrl}:`, err.message);

  let statusCode = err.statusCode || err.status || 500;

  // Map known errors to appropriate HTTP status codes
  if (err.name === "ValidationError" || err.message?.includes("Validation") || err.message?.includes("Compilation")) {
    statusCode = 400;
  } else if (err.message?.includes("not found")) {
    statusCode = 404;
  } else if (err.message?.includes("unavailable") || err.message?.includes("offline")) {
    statusCode = 503;
  }

  res.status(statusCode).json({
    success: false,
    error: {
      status: statusCode,
      message: err.message || "Internal Server Error",
      ...(process.env.NODE_ENV === "development" ? { stack: err.stack } : {})
    }
  });
};
