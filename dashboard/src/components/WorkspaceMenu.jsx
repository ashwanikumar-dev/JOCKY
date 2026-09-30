import React, { useEffect, useRef } from "react";

export function WorkspaceMenu({ isOpen, onClose, activeWorkspace, onSelectWorkspace }) {
  const menuRef = useRef(null);

  useEffect(() => {
    const handleClickOutside = (e) => {
      if (menuRef.current && !menuRef.current.contains(e.target)) {
        onClose();
      }
    };
    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div ref={menuRef} id="menu-dropdown" className="menu-dropdown">
      <div className="menu-main-nav">
        {/* 1. Dashboard */}
        <button
          className={`menu-item ${!activeWorkspace ? "active" : ""}`}
          onClick={() => {
            onSelectWorkspace(null);
            onClose();
          }}
        >
          <div className="menu-item-left">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="3" width="7" height="7" />
              <rect x="14" y="3" width="7" height="7" />
              <rect x="14" y="14" width="7" height="7" />
              <rect x="3" y="14" width="7" height="7" />
            </svg>
            <span>Dashboard</span>
          </div>
        </button>

        {/* 2. Investigation */}
        <button
          className={`menu-item ${activeWorkspace === "modal-investigation" ? "active" : ""}`}
          onClick={() => {
            onSelectWorkspace("modal-investigation");
            onClose();
          }}
        >
          <div className="menu-item-left">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="11" cy="11" r="8" />
              <path d="m21 21-4.35-4.35" />
            </svg>
            <span>Investigation</span>
          </div>
        </button>

        {/* 3. Machines */}
        <button
          className={`menu-item ${activeWorkspace === "modal-machines" || activeWorkspace === "modal-machine-detail" ? "active" : ""}`}
          onClick={() => {
            onSelectWorkspace("modal-machines");
            onClose();
          }}
        >
          <div className="menu-item-left">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="2" y="3" width="20" height="14" rx="2" />
              <path d="M8 21h8M12 17v4" />
            </svg>
            <span>Machines</span>
          </div>
        </button>

        {/* 4. Evidence */}
        <button
          className={`menu-item ${activeWorkspace === "modal-evidence" ? "active" : ""}`}
          onClick={() => {
            onSelectWorkspace("modal-evidence");
            onClose();
          }}
        >
          <div className="menu-item-left">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <ellipse cx="12" cy="5" rx="9" ry="3" />
              <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" />
              <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" />
            </svg>
            <span>Evidence</span>
          </div>
        </button>

        {/* 5. Findings */}
        <button
          className={`menu-item ${activeWorkspace === "modal-findings" ? "active" : ""}`}
          onClick={() => {
            onSelectWorkspace("modal-findings");
            onClose();
          }}
        >
          <div className="menu-item-left">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            </svg>
            <span>Findings</span>
          </div>
        </button>

        {/* 6. Timeline */}
        <button
          className={`menu-item ${activeWorkspace === "modal-timeline" ? "active" : ""}`}
          onClick={() => {
            onSelectWorkspace("modal-timeline");
            onClose();
          }}
        >
          <div className="menu-item-left">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="10" />
              <polyline points="12 6 12 12 16 14" />
            </svg>
            <span>Timeline</span>
          </div>
        </button>

        {/* 7. Reports */}
        <button
          className={`menu-item ${activeWorkspace === "modal-audit-custody" ? "active" : ""}`}
          onClick={() => {
            onSelectWorkspace("modal-audit-custody");
            onClose();
          }}
          aria-label="Reports"
          title="Reports"
        >
          <div className="menu-item-left">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
              <line x1="16" y1="13" x2="8" y2="13" />
              <line x1="16" y1="17" x2="8" y2="17" />
              <polyline points="10 9 9 9 8 9" />
            </svg>
            <span>Reports</span>
          </div>
        </button>
      </div>

      {/* Bottom Settings Item */}
      <div className="menu-bottom-nav">
        <button
          className={`menu-item menu-item-settings ${activeWorkspace === "modal-settings" ? "active" : ""}`}
          onClick={() => {
            onSelectWorkspace("modal-settings");
            onClose();
          }}
          aria-label="Settings"
          title="Settings"
        >
          <div className="menu-item-left">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
            </svg>
            <span>Settings</span>
          </div>
        </button>
      </div>
    </div>
  );
}
