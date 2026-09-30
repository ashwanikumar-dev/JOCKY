// Compiler Service
// Uses compiler adapter to process JOCKY script into validated AST and Forensic IR

const compilerAdapter = require("../adapters/compiler.adapter");

module.exports = {
  compile: async (script) => {
    return await compilerAdapter.compileScript(script);
  }
};
