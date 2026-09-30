import React from "react";
import { Modal } from "../components/Modal";
import { OsIcon } from "../utils/osIcon";

export function MachineDetailModal({
  isOpen,
  onClose,
  machine,
  onBackToMachines,
  onStartInvestigation,
  onViewEvidence,
  onViewFindings
}) {
  if (!machine) return null;

  const {
    hostname = "WIN-FORENSIC-01",
    ip = "10.0.1.47",
    os = "Windows 11 Pro (22631)",
    agent = "v0.4.2",
    heartbeat = "20:42:27 UTC",
    arch = "x86_64",
    status = "ONLINE",
    evidenceCount = 18,
    findingsCount = 1,
    processes = 148,
    connections = 24,
    activity = []
  } = machine;

  const findingsColor = findingsCount > 0 ? "var(--red)" : "var(--green)";

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      id="modal-machine-detail"
      title={hostname}
      subtitle="Endpoint Detailed Telemetry & Forensics"
      icon={
        <div id="md-title-os-icon" className="mcard-os-icon" style={{ width: "32px", height: "32px" }}>
          <OsIcon osName={os} size={28} />
        </div>
      }
    >
      <div className="machine-detail-layout">
        {/* Back navigation & Actions row */}
        <div className="detail-nav-row">
          <button className="btn-back-machines" onClick={onBackToMachines}>
            <span>← Back to Machines</span>
          </button>

          <div className="detail-actions-group">
            <button className="btn-secondary" id="btn-md-investigate" onClick={onStartInvestigation}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="11" cy="11" r="8" />
                <path d="m21 21-4.35-4.35" />
              </svg>
              <span>Start Investigation</span>
            </button>
            <button className="btn-tertiary" onClick={onViewEvidence}>
              <span>View Evidence</span>
            </button>
            <button className="btn-tertiary" onClick={onViewFindings}>
              <span>View Findings</span>
            </button>
          </div>
        </div>

        {/* Quick Stats Row */}
        <div className="quick-stats-row">
          <div className="quick-stat-box">
            <div className="quick-stat-num" id="md-stat-evidence">
              {evidenceCount}
            </div>
            <div className="quick-stat-lbl">Preserved Evidence</div>
          </div>
          <div className="quick-stat-box">
            <div className="quick-stat-num" id="md-stat-findings" style={{ color: findingsColor }}>
              {findingsCount}
            </div>
            <div className="quick-stat-lbl">Security Findings</div>
          </div>
          <div className="quick-stat-box">
            <div className="quick-stat-num" id="md-stat-processes">
              {processes}
            </div>
            <div className="quick-stat-lbl">Active Processes</div>
          </div>
          <div className="quick-stat-box">
            <div className="quick-stat-num" id="md-stat-connections">
              {connections}
            </div>
            <div className="quick-stat-lbl">Network Connections</div>
          </div>
        </div>

        {/* Detail Grid: Metadata & Recent Activity */}
        <div className="detail-grid">
          {/* Metadata Card */}
          <div className="detail-card">
            <div className="detail-card-head">
              <h4>Endpoint Information</h4>
              <span className="tag tag-success" id="md-tag-status">
                ● {status}
              </span>
            </div>
            <div className="detail-card-body">
              <div className="meta-key-val-grid">
                <div className="meta-kv-item">
                  <span className="meta-key">Hostname</span>
                  <span className="meta-val" id="md-val-hostname">{hostname}</span>
                </div>
                <div className="meta-kv-item">
                  <span className="meta-key">IP Address</span>
                  <span className="meta-val" id="md-val-ip">{ip}</span>
                </div>
                <div className="meta-kv-item">
                  <span className="meta-key">Operating System</span>
                  <div style={{ display: "flex", alignItems: "center", gap: "6px", marginTop: "2px" }}>
                    <span id="md-info-os-icon">
                      <OsIcon osName={os} size={16} />
                    </span>
                    <span className="meta-val" id="md-val-os" style={{ marginTop: 0 }}>
                      {os}
                    </span>
                  </div>
                </div>
                <div className="meta-kv-item">
                  <span className="meta-key">Agent Version</span>
                  <span className="meta-val" id="md-val-agent">{agent}</span>
                </div>
                <div className="meta-kv-item">
                  <span className="meta-key">Last Heartbeat</span>
                  <span className="meta-val" id="md-val-heartbeat">{heartbeat}</span>
                </div>
                <div className="meta-kv-item">
                  <span className="meta-key">Architecture</span>
                  <span className="meta-val" id="md-val-arch">{arch}</span>
                </div>
              </div>
            </div>
          </div>

          {/* Recent Activity Card */}
          <div className="detail-card">
            <div className="detail-card-head">
              <h4>Recent Node Activity</h4>
              <span className="pill-label">Live Audit</span>
            </div>
            <div className="detail-card-body">
              <div className="activity-list" id="md-activity-container">
                {activity && activity.length > 0 ? (
                  activity.map((act, idx) => (
                    <div className="activity-item" key={idx}>
                      <span>{act.text}</span>
                      {act.tag ? (
                        <span className="tag tag-high" style={{ fontSize: "9px", padding: "1px 4px" }}>
                          {act.tag}
                        </span>
                      ) : (
                        <span className="mono-small" style={{ color: "var(--text-dim)" }}>
                          {act.time}
                        </span>
                      )}
                    </div>
                  ))
                ) : (
                  <div className="activity-item">
                    <span>Node telemetry connected</span>
                    <span className="mono-small" style={{ color: "var(--text-dim)" }}>Live</span>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </Modal>
  );
}
