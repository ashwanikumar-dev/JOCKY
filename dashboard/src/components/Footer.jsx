import React from "react";

export function Footer({ nodeCount = 4 }) {
  return (
    <footer className="footer">
      <div className="footer-left">
        <img src="/assets/jocky-mark.png" alt="JOCKY" className="footer-logo-img" />
        <strong>JOCKY IR Console</strong> <span className="footer-sep">|</span> v1.0.0
      </div>
      <div className="footer-right">
        <span className="status-dot pulse" />
        <span className="mono-small">{nodeCount} Enrolled Agent Nodes · UTC Normalized</span>
      </div>
    </footer>
  );
}
