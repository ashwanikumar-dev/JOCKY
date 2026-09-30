const express = require("express");
const cors = require("cors");
const routes = require("./routes");
const notFoundMiddleware = require("./middleware/notFound.middleware");
const errorMiddleware = require("./middleware/error.middleware");
const env = require("./config/env");
const logger = require("./utils/logger");

const app = express();

// Configure CORS for dashboard
app.use(cors({
  origin: env.corsOrigin === "*" ? "*" : [env.corsOrigin, "http://localhost:5173", "http://127.0.0.1:5173"],
  credentials: true,
  methods: ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"]
}));

// Body parsers
app.use(express.json({ limit: "10mb" }));
app.use(express.urlencoded({ extended: true, limit: "10mb" }));

// Request logging
app.use((req, res, next) => {
  logger.info(`${req.method} ${req.originalUrl}`);
  next();
});

// Mount Handbook-driven API routes
app.use("/api", routes);

// 404 handler
app.use(notFoundMiddleware);

// Global Error Handler
app.use(errorMiddleware);

module.exports = app;
