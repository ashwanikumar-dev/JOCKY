const express = require("express");
const router = express.Router();
const evidenceController = require("../controllers/evidence.controller");

router.get("/", evidenceController.getAll);
router.get("/:id", evidenceController.getById);
router.post("/", evidenceController.preserve);

module.exports = router;
