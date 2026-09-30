import React, { useState } from "react";
import { WorkspaceMenu } from "./WorkspaceMenu";

export function Subbar({ activeWorkspace, onSelectWorkspace, activeCaseTitle }) {
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <div className="workbench-subbar">
      <div className="menu-btn-container">
        <button
          id="btn-main-menu"
          className={`hamburger-btn ${menuOpen ? "active" : ""}`}
          aria-label="Open Workspaces Menu"
          title="Workspaces Menu"
          onClick={() => setMenuOpen(!menuOpen)}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
            <line x1="3" y1="6" x2="21" y2="6" />
            <line x1="3" y1="12" x2="21" y2="12" />
            <line x1="3" y1="18" x2="21" y2="18" />
          </svg>
        </button>

        <WorkspaceMenu
          isOpen={menuOpen}
          onClose={() => setMenuOpen(false)}
          activeWorkspace={activeWorkspace}
          onSelectWorkspace={onSelectWorkspace}
        />
      </div>

      <div className="subbar-context">
        <span className="subbar-tag">ACTIVE CASE</span>
        <span className="subbar-title">
          {activeCaseTitle || "Endpoint Triage #IR-2026-0042 · Enrolled Fleet (4 Nodes Online)"}
        </span>
      </div>
    </div>
  );
}
