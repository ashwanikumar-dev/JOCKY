// Central Router for JOCKY Dashboard Backend
// Maps to Handbook Contract #10 (Backend -> Dashboard API)

const express = require("express");
const router = express.Router();

const healthRoutes = require("./health.routes");
const dashboardRoutes = require("./dashboard.routes");
const investigationsRoutes = require("./investigations.routes");
const machinesRoutes = require("./machines.routes");
const evidenceRoutes = require("./evidence.routes");
const findingsRoutes = require("./findings.routes");
const timelineRoutes = require("./timeline.routes");
const auditRoutes = require("./audit.routes");
const compileRoutes = require("./compile.routes");

// Health check endpoint
router.use("/health", healthRoutes);

// Dashboard Aggregates & KPIs
router.use("/dashboard", dashboardRoutes);

// Handbook Core Contract Endpoints
router.use("/investigations", investigationsRoutes);
router.use("/machines", machinesRoutes);
router.use("/evidence", evidenceRoutes);
router.use("/findings", findingsRoutes);
router.use("/timeline", timelineRoutes);
router.use("/audit", auditRoutes);
router.use("/compile", compileRoutes);

module.exports = router;
