const express = require("express");
const router = express.Router();
const findingsController = require("../controllers/findings.controller");

router.get("/", findingsController.getAll);
router.get("/:id", findingsController.getById);
router.post("/", findingsController.create);

module.exports = router;
