import React from "react";
import { OsIcon } from "../utils/osIcon";

export function MachineCard({ machine, onSelect }) {
  const {
    id,
    hostname,
    os,
    status = "ONLINE",
    evidenceCount = 0,
    findingsCount = 0,
    processes = 0,
    ip = "10.0.1.0",
    heartbeat = "20:42 UTC"
  } = machine;

  const findingsColor =
    findingsCount > 0
      ? hostname.includes("UBUNTU")
        ? "var(--amber)"
        : "var(--red)"
      : "var(--text-main)";

  const findingsLabel =
    findingsCount > 0
      ? hostname.includes("UBUNTU")
        ? `${findingsCount} Med`
        : `${findingsCount} High`
      : "0";

  return (
    <div className="machine-card" onClick={() => onSelect(id || hostname)}>
      <div className="mcard-head">
        <div className="mcard-device-info">
          <div className="mcard-os-icon">
            <OsIcon osName={os} size={18} />
          </div>
          <div className="mcard-name-wrap">
            <span className="mcard-hostname">{hostname}</span>
            <span className="mcard-os">{os}</span>
          </div>
        </div>
        <span className={`tag ${status === "ONLINE" ? "tag-success" : "tag-info"}`}>
          ● {status}
        </span>
      </div>

      <div className="mcard-stats-row">
        <div className="mcard-stat-item">
          <span className="mcard-stat-label">Evidence</span>
          <span className="mcard-stat-val">{evidenceCount}</span>
        </div>
        <div className="mcard-stat-item">
          <span className="mcard-stat-label">Findings</span>
          <span className="mcard-stat-val" style={{ color: findingsColor }}>
            {findingsLabel}
          </span>
        </div>
        <div className="mcard-stat-item">
          <span className="mcard-stat-label">Processes</span>
          <span className="mcard-stat-val">{processes}</span>
        </div>
      </div>

      <div className="mcard-footer">
        <span className="mcard-heartbeat">IP: {ip} · {heartbeat}</span>
        <button
          className="btn-view-details"
          onClick={(e) => {
            e.stopPropagation();
            onSelect(id || hostname);
          }}
        >
          View Details →
        </button>
      </div>
    </div>
  );
}
