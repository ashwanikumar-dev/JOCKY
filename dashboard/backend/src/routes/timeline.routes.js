const express = require("express");
const router = express.Router();
const timelineController = require("../controllers/timeline.controller");

router.get("/", timelineController.getAll);
router.post("/", timelineController.create);

module.exports = router;
