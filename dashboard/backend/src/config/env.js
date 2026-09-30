const dotenv = require("dotenv");
dotenv.config();

module.exports = {
  port: process.env.PORT || 8001,
  nodeEnv: process.env.NODE_ENV || "development",
  corsOrigin: process.env.CORS_ORIGIN || "http://localhost:5173",
  compilerPath: process.env.COMPILER_PATH || "../../compiler",
  m5BaseUrl: process.env.M5_BASE_URL || "http://127.0.0.1:8000",
};