const express = require("express");
const router = express.Router();
const investigationsController = require("../controllers/investigations.controller");

router.get("/", investigationsController.getAll);

router.post("/:id/run", investigationsController.run);

router.get("/:id", investigationsController.getById);

router.post("/", investigationsController.create);

module.exports = router;
