// JOCKY Compiler Adapter
// Connects Product Backend directly to the REAL M3 Rust compiler.

const path = require("path");
const fs = require("fs");
const os = require("os");
const { spawnSync } = require("child_process");
const logger = require("../utils/logger");

const COMPILER_BINARY_PATH =
  "C:\\JOCKEY\\mix m3 m4\\compiler\\target\\debug\\compiler.exe";

/**
 * Compile JOCKY DSL using the real M3 compiler.
 *
 * M3 CLI contract:
 *   compiler.exe <script-file> <investigation-id>
 *
 * The compiler receives a temporary .jocky source file and returns
 * the final validated Forensic IR JSON.
 */
module.exports = {
  integrationStatus: "REAL M3 COMPILER INTEGRATION",

  compileScript: async (script, investigationId) => {
    const raw = (script || "").trim();

    if (!raw) {
      const err = new Error(
        "Compilation Error: JOCKY script source cannot be empty.",
      );
      err.statusCode = 400;
      throw err;
    }

    if (!fs.existsSync(COMPILER_BINARY_PATH)) {
      const err = new Error(
        `M3 compiler not found at: ${COMPILER_BINARY_PATH}`,
      );
      err.statusCode = 500;
      throw err;
    }

    const id = investigationId || `inv-${Date.now()}`;

    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "jocky-compiler-"));

    const scriptPath = path.join(tempDir, `${id}.jocky`);

    try {
      fs.writeFileSync(scriptPath, raw, "utf8");

      const proc = spawnSync(COMPILER_BINARY_PATH, [scriptPath, id], {
        encoding: "utf8",
        timeout: 10000,
        maxBuffer: 10 * 1024 * 1024,
        windowsHide: true,
      });

      if (proc.error) {
        const err = new Error(
          `M3 compiler execution failed: ${proc.error.message}`,
        );
        err.statusCode = 500;
        throw err;
      }

      const stdout = (proc.stdout || "").trim();
      const stderr = (proc.stderr || "").trim();

      if (proc.status !== 0) {
        logger.error("M3 compiler returned non-zero exit code", {
          status: proc.status,
          stderr,
          stdout,
        });

        let message = stderr || stdout || "M3 compilation failed.";

        try {
          const parsedError = JSON.parse(stdout);
          message = parsedError.error || parsedError.message || message;
        } catch {
          // Keep raw compiler output.
        }

        const err = new Error(message);
        err.statusCode = 400;
        throw err;
      }

      if (!stdout) {
        const err = new Error("M3 compiler returned an empty response.");
        err.statusCode = 500;
        throw err;
      }

      let ir;

      try {
        const jsonStart = stdout.indexOf("{");
        const jsonEnd = stdout.lastIndexOf("}");

        if (jsonStart === -1 || jsonEnd === -1) {
          throw new Error("No JSON object found in compiler output.");
        }

        ir = JSON.parse(stdout.slice(jsonStart, jsonEnd + 1));
      } catch (parseError) {
        logger.error("Failed to parse M3 compiler JSON output", {
          stdout,
          stderr,
          error: parseError.message,
        });

        const err = new Error("M3 compiler returned invalid JSON.");
        err.statusCode = 500;
        throw err;
      }

      return {
        ast: null,
        ir,
        valid: true,
        compiler_source: "REAL_M3_COMPILER",
      };
    } finally {
      try {
        fs.rmSync(tempDir, {
          recursive: true,
          force: true,
        });
      } catch (cleanupError) {
        logger.warn("Failed to clean compiler temp directory", {
          error: cleanupError.message,
        });
      }
    }
  },
};
