import React from "react";
import { Modal } from "../components/Modal";

export function SettingsModal({ isOpen, onClose, isOnline }) {
  const apiBase = import.meta.env.VITE_API_BASE_URL || "http://localhost:8000";

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      id="modal-settings"
      title="Settings &amp; Environment"
      subtitle="JOCKY Incident Response Console Configuration"
      icon={
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
      }
    >
      <div className="detail-card">
        <div className="detail-card-head">
          <h4>Backend Connectivity &amp; Architecture</h4>
          <span className={`tag ${isOnline ? "tag-success" : "tag-high"}`}>
            ● {isOnline ? "ONLINE" : "DISCONNECTED"}
          </span>
        </div>
        <div className="detail-card-body">
          <div className="meta-key-val-grid">
            <div className="meta-kv-item">
              <span className="meta-key">API Base URL</span>
              <span className="meta-val mono-small">{apiBase}</span>
            </div>
            <div className="meta-kv-item">
              <span className="meta-key">Environment Mode</span>
              <span className="meta-val">Development</span>
            </div>
            <div className="meta-kv-item">
              <span className="meta-key">Theme</span>
              <span className="meta-val">Enterprise Light (Forensic)</span>
            </div>
            <div className="meta-kv-item">
              <span className="meta-key">Hash Integrity Algo</span>
              <span className="meta-val">SHA-256 (Enforced)</span>
            </div>
            <div className="meta-kv-item">
              <span className="meta-key">DSL Specification</span>
              <span className="meta-val">JOCKY Grammar v0.1</span>
            </div>
            <div className="meta-kv-item">
              <span className="meta-key">Telemetry Poll</span>
              <span className="meta-val">Real-time (UTC Synchronized)</span>
            </div>
          </div>
        </div>
      </div>
    </Modal>
  );
}
