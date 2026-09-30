import React from "react";

export function PipelineStatus({ status, step = "idle" }) {
  // Normalize step if only status is passed
  const currentStep = step !== "idle" ? step : status === "COMPLETE" ? "complete" : status === "COMPILING" ? "parsing" : "idle";

  const isComplete = currentStep === "complete";
  const isError = currentStep === "error";

  // Helpers for Pipeline Row boxes
  const getBoxProps = (stage) => {
    switch (stage) {
      case "lexer":
        if (currentStep === "error") return { className: "pipe-box error", text: "ERROR" };
        if (currentStep === "parsing") return { className: "pipe-box active", text: "RUNNING..." };
        if (["ast", "ir", "complete"].includes(currentStep)) return { className: "pipe-box complete", text: "COMPLETE" };
        return { className: "pipe-box", text: "READY" };

      case "parser":
        if (currentStep === "error") return { className: "pipe-box error", text: "FAILED" };
        if (currentStep === "parsing") return { className: "pipe-box active", text: "PARSING..." };
        if (["ast", "ir", "complete"].includes(currentStep)) return { className: "pipe-box complete", text: "COMPLETE" };
        return { className: "pipe-box", text: "READY" };

      case "ast":
        if (currentStep === "error") return { className: "pipe-box error", text: "ABORTED" };
        if (currentStep === "parsing") return { className: "pipe-box", text: "PENDING" };
        if (currentStep === "ast") return { className: "pipe-box active", text: "BUILDING..." };
        if (["ir", "complete"].includes(currentStep)) return { className: "pipe-box complete", text: "COMPLETE" };
        return { className: "pipe-box", text: "READY" };

      case "ir":
        if (currentStep === "error") return { className: "pipe-box error", text: "ABORTED" };
        if (["parsing", "ast"].includes(currentStep)) return { className: "pipe-box", text: "PENDING" };
        if (currentStep === "ir") return { className: "pipe-box active", text: "EMITTING..." };
        if (currentStep === "complete") return { className: "pipe-box complete", text: "COMPLETE" };
        return { className: "pipe-box", text: "READY" };

      default:
        return { className: "pipe-box", text: "READY" };
    }
  };

  const lexerBox = getBoxProps("lexer");
  const parserBox = getBoxProps("parser");
  const astBox = getBoxProps("ast");
  const irBox = getBoxProps("ir");

  return (
    <div className="val-card">
      <div className="panel-card-head">
        <span className="panel-card-title">VALIDATION &amp; PIPELINE</span>
        <span
          id="val-badge-overall"
          className={`tag ${
            isComplete
              ? "tag-success"
              : isError
              ? "tag-error"
              : currentStep !== "idle"
              ? "tag-medium"
              : "tag-info"
          }`}
          style={isError ? { background: "#fef2f2", color: "#dc2626", border: "1px solid #fecaca" } : {}}
        >
          {isComplete ? (
            "VALIDATED"
          ) : isError ? (
            "ERROR"
          ) : currentStep === "parsing" ? (
            <>
              <span className="step-dot-pulse" />
              PARSING...
            </>
          ) : currentStep === "ast" ? (
            <>
              <span className="step-dot-pulse" />
              BUILDING AST...
            </>
          ) : currentStep === "ir" ? (
            <>
              <span className="step-dot-pulse" />
              BUILDING IR...
            </>
          ) : (
            "READY"
          )}
        </span>
      </div>

      <div className="panel-card-body">
        <div className="val-checklist">
          {/* Item 1: Syntax Check */}
          <div className={`val-item ${currentStep === "parsing" ? "active" : isError ? "error" : ""}`}>
            <div className="val-item-left">
              {currentStep === "parsing" ? (
                <>
                  <span className="step-dot-pulse" />
                  <span>Parsing JOCKY DSL...</span>
                </>
              ) : isError ? (
                <>
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#dc2626" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                  </svg>
                  <span style={{ color: "#dc2626" }}>Syntax Check</span>
                </>
              ) : (
                <>
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                  <span>{["ast", "ir", "complete"].includes(currentStep) ? "✓ Syntax Check" : "Syntax Check"}</span>
                </>
              )}
            </div>
            <span
              id="val-check-syntax"
              className={`val-status-tag ${
                isError
                  ? "val-error"
                  : currentStep === "parsing"
                  ? "val-active"
                  : "val-pass"
              }`}
            >
              {isError ? "Syntax Error" : currentStep === "parsing" ? "Lexing & Parsing..." : "Grammar v0.1 Valid"}
            </span>
          </div>

          {/* Item 2: AST Generation */}
          <div className={`val-item ${currentStep === "ast" ? "active" : ""}`}>
            <div className="val-item-left">
              {currentStep === "ast" ? (
                <>
                  <span className="step-dot-pulse" />
                  <span>Building AST...</span>
                </>
              ) : ["ir", "complete"].includes(currentStep) ? (
                <>
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                  <span>✓ AST Generation</span>
                </>
              ) : (
                <>
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <polyline points="16 18 22 12 16 6" />
                    <polyline points="8 6 2 12 8 18" />
                  </svg>
                  <span>AST Generation</span>
                </>
              )}
            </div>
            <span
              id="val-check-ast"
              className={`val-status-tag ${
                isError
                  ? "val-wait"
                  : ["ir", "complete"].includes(currentStep)
                  ? "val-pass"
                  : currentStep === "ast"
                  ? "val-active"
                  : "val-wait"
              }`}
            >
              {isError
                ? "Aborted"
                : ["ir", "complete"].includes(currentStep)
                ? "ast_v1_contract (OK)"
                : currentStep === "ast"
                ? "Building AST..."
                : "Pending Compile"}
            </span>
          </div>

          {/* Item 3: Forensic IR / Compiler Status */}
          <div className={`val-item ${currentStep === "ir" ? "active" : ""}`}>
            <div className="val-item-left">
              {currentStep === "ir" ? (
                <>
                  <span className="step-dot-pulse" />
                  <span>Building Forensic IR...</span>
                </>
              ) : isComplete ? (
                <>
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                  <span>✓ Forensic IR Generated</span>
                </>
              ) : (
                <>
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <circle cx="12" cy="12" r="3" />
                    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
                  </svg>
                  <span>Compiler Status</span>
                </>
              )}
            </div>
            <span
              id="val-check-status"
              className={`val-status-tag ${
                isError
                  ? "val-wait"
                  : isComplete
                  ? "val-pass"
                  : currentStep === "ir"
                  ? "val-active"
                  : "val-wait"
              }`}
            >
              {isError
                ? "Aborted"
                : isComplete
                ? "IR Emitted (OK)"
                : currentStep === "ir"
                ? "Emitting IR..."
                : "Idle"}
            </span>
          </div>
        </div>

        {/* Pipeline Stages */}
        <div className="pipeline-row">
          <div className={lexerBox.className} id="pipe-lexer">
            <span className="pipe-box-name">LEXER</span>
            <span className="pipe-box-status">{lexerBox.text}</span>
          </div>
          <div className={parserBox.className} id="pipe-parser">
            <span className="pipe-box-name">PARSER</span>
            <span className="pipe-box-status">{parserBox.text}</span>
          </div>
          <div className={astBox.className} id="pipe-ast">
            <span className="pipe-box-name">AST GEN</span>
            <span className="pipe-box-status">{astBox.text}</span>
          </div>
          <div className={irBox.className} id="pipe-ir">
            <span className="pipe-box-name">FORENSIC IR</span>
            <span className="pipe-box-status">{irBox.text}</span>
          </div>
        </div>
      </div>
    </div>
  );
}
