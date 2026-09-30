const app = require("./app");
const env = require("./config/env");
const logger = require("./utils/logger");

const server = app.listen(env.port, () => {
  logger.info(`=======================================================`);
  logger.info(`  JOCKY Dashboard Backend Service (Handbook Compliant)  `);
  logger.info(`  Environment: ${env.nodeEnv}`);
  logger.info(`  Port:        ${env.port}`);
  logger.info(`  Health:      http://localhost:${env.port}/api/health`);
  logger.info(`  Dashboard:   ${env.corsOrigin}`);
  logger.info(`=======================================================`);
});

// Graceful shutdown
process.on("SIGTERM", () => {
  logger.info("SIGTERM received, shutting down gracefully");
  server.close(() => process.exit(0));
});

process.on("SIGINT", () => {
  logger.info("SIGINT received, shutting down gracefully");
  server.close(() => process.exit(0));
});
