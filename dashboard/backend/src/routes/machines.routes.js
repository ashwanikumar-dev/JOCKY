const express = require("express");
const router = express.Router();
const machinesController = require("../controllers/machines.controller");

router.get("/", machinesController.getAll);
router.get("/:id", machinesController.getById);
router.post("/", machinesController.register);

module.exports = router;
