const express = require("express");
const router = express.Router();
const compileController = require("../controllers/compile.controller");

router.post("/", compileController.compile);

module.exports = router;
