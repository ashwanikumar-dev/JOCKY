import React, { useEffect } from "react";

export function Modal({ isOpen, onClose, id, title, subtitle, icon, badge, children, headerRight }) {
  useEffect(() => {
    if (isOpen) {
      document.body.classList.add("modal-open");
    } else {
      document.body.classList.remove("modal-open");
    }
    return () => {
      document.body.classList.remove("modal-open");
    };
  }, [isOpen]);

  if (!isOpen) return null;

  return (
    <div id={id} className="workspace-modal" role="dialog" aria-modal="true">
      <div className="modal-backdrop" onClick={onClose} />
      <div className="workspace-modal-box">
        <div className="workspace-modal-header">
          <div className="modal-title-wrap">
            {icon}
            <div>
              <h3>{title}</h3>
              {subtitle && (
                <span style={{ fontSize: "11px", color: "var(--text-muted)", display: "block", fontWeight: "normal" }}>
                  {subtitle}
                </span>
              )}
            </div>
            {badge}
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            {headerRight}
            <button className="modal-close-btn" onClick={onClose} aria-label="Close modal">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
        </div>
        <div className="workspace-modal-body">{children}</div>
      </div>
    </div>
  );
}
