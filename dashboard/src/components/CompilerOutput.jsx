import React, { useState } from "react";

export function CompilerOutput({ compiledIR, ast, activeTab, onTabChange }) {
  const [copied, setCopied] = useState(false);

  const getOutputText = () => {
    if (activeTab === "ir") {
      return compiledIR
        ? JSON.stringify(compiledIR, null, 2)
        : '// Click "Compile Script" to inspect AST & Forensic IR';
    } else {
      return ast
        ? JSON.stringify(ast, null, 2)
        : "// AST representation generated upon compilation";
    }
  };

  const handleCopy = () => {
    const text = getOutputText();
    navigator.clipboard?.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  return (
    <div className="output-card">
      <div className="panel-card-head">
        <div className="output-tab-bar">
          <button
            id="tab-btn-ir"
            className={`out-tab-btn ${activeTab === "ir" ? "active" : ""}`}
            onClick={() => onTabChange("ir")}
          >
            Forensic IR
          </button>
          <button
            id="tab-btn-ast"
            className={`out-tab-btn ${activeTab === "ast" ? "active" : ""}`}
            onClick={() => onTabChange("ast")}
          >
            AST Contract v1.0
          </button>
        </div>

        <button
          className={`copy-btn ${copied ? "copied" : ""}`}
          id="btn-copy-output"
          onClick={handleCopy}
          title="Copy output to clipboard"
        >
          <span className="copy-icon-wrap">
            {copied ? (
              <svg
                className="copy-svg check-svg"
                width="13"
                height="13"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2.2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
            ) : (
              <svg
                className="copy-svg"
                width="13"
                height="13"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </svg>
            )}
          </span>
          <span className="copy-label">{copied ? "Copied" : "Copy"}</span>
        </button>
      </div>

      <div className="panel-card-body" style={{ padding: "10px" }}>
        <pre id="output-code-view" className="output-code-container">
          {getOutputText()}
        </pre>
      </div>
    </div>
  );
}
