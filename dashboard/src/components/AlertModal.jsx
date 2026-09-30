import React from "react";

export function AlertModal({
  isOpen,
  onClose,
  type = "success",
  title,
  children,
}) {
  if (!isOpen) return null;

  const isError = type === "error";

  return (
    <div className="jocky-alert-overlay" onClick={onClose}>
      <div
        className={`jocky-alert-card jocky-alert-${type}`}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="jocky-alert-header">
          <div className="jocky-alert-icon">{isError ? "!" : "✓"}</div>

          <div>
            <h3>{title}</h3>
            <span>
              {isError
                ? "Operation failed"
                : "Operation completed successfully"}
            </span>
          </div>

          <button
            className="jocky-alert-close"
            onClick={onClose}
            aria-label="Close"
          >
            ×
          </button>
        </div>

        <div className="jocky-alert-body">{children}</div>

        <div className="jocky-alert-footer">
          <button className="btn-primary" onClick={onClose}>
            Continue
          </button>
        </div>
      </div>
    </div>
  );
}
