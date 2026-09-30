import React from "react";

export function DashboardView({
  stats,
  investigations = [],
  findings = [],
  onOpenWorkspace,
  onStartInvestigation,
}) {
  const kpi = stats?.kpi || {
    activeInvestigation: {
      title: "Active Investigation",
      value: "No Active Investigation",
      sub: "No investigation created",
    },

    enrolledEndpoints: {
      title: "Enrolled Endpoints",
      value: "0 / 0 Online",
      healthBadge: "NO NODES",
      sub: "0 Windows · 0 Linux",
    },

    preservedEvidence: {
      title: "Preserved Evidence",
      value: "0 Artifacts",
      hashBadge: "SHA-256",
      sub: "Tamper-evident chain",
    },

    securityFindings: {
      title: "Security Findings",
      value: "0 Detections",
      highCount: 0,
      sub: "0 Medium · 0 Critical",
    },
  };

  return (
    <main className="dashboard-main">
      {/* Welcome / Case Status Card */}
      <div className="dash-welcome-card">
        <div className="dash-welcome-text">
          <h2>Welcome back, Investigator</h2>
          <p>
            Active forensic investigation console for enrolled fleet telemetry,
            evidence chain of custody, and DSL triage.
          </p>
        </div>
        <div className="dash-welcome-actions">
          <button
            className="btn-primary"
            onClick={() => onOpenWorkspace("modal-investigation")}
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <circle cx="11" cy="11" r="8" />
              <path d="m21 21-4.35-4.35" />
            </svg>
            <span>Open Investigation IDE</span>
          </button>
        </div>
      </div>

      {/* KPI Summary Row (4 Compact Cards) */}
      <div className="dash-kpi-grid">
        {/* KPI 1 */}
        <div
          className="kpi-card"
          onClick={() => onOpenWorkspace("modal-investigation")}
          title="Click to open Investigation workspace"
        >
          <div className="kpi-head">
            <span className="kpi-title">{kpi.activeInvestigation.title}</span>
            <svg
              className="kpi-icon"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <circle cx="11" cy="11" r="8" />
              <path d="m21 21-4.35-4.35" />
            </svg>
          </div>
          <div className="kpi-value">{kpi.activeInvestigation.value}</div>
          <div className="kpi-sub">
            <span className="status-dot" />
            <span>{kpi.activeInvestigation.sub}</span>
          </div>
        </div>

        {/* KPI 2 */}
        <div
          className="kpi-card"
          onClick={() => onOpenWorkspace("modal-machines")}
          title="Click to open Machines workspace"
        >
          <div className="kpi-head">
            <span className="kpi-title">{kpi.enrolledEndpoints.title}</span>
            <svg
              className="kpi-icon"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <rect x="2" y="3" width="20" height="14" rx="2" />
              <path d="M8 21h8M12 17v4" />
            </svg>
          </div>
          <div className="kpi-value">{kpi.enrolledEndpoints.value}</div>
          <div className="kpi-sub">
            <span
              className="tag tag-success"
              style={{ fontSize: "9.5px", padding: "1px 5px" }}
            >
              {kpi.enrolledEndpoints.healthBadge || "100% HEALTH"}
            </span>
            <span>{kpi.enrolledEndpoints.sub}</span>
          </div>
        </div>

        {/* KPI 3 */}
        <div
          className="kpi-card"
          onClick={() => onOpenWorkspace("modal-evidence")}
          title="Click to open Evidence workspace"
        >
          <div className="kpi-head">
            <span className="kpi-title">{kpi.preservedEvidence.title}</span>
            <svg
              className="kpi-icon"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <ellipse cx="12" cy="5" rx="9" ry="3" />
              <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" />
              <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" />
            </svg>
          </div>
          <div className="kpi-value">{kpi.preservedEvidence.value}</div>
          <div className="kpi-sub">
            <span
              className="tag tag-success"
              style={{ fontSize: "9.5px", padding: "1px 5px" }}
            >
              {kpi.preservedEvidence.hashBadge || "SHA-256"}
            </span>
            <span>{kpi.preservedEvidence.sub}</span>
          </div>
        </div>

        {/* KPI 4 */}
        <div
          className="kpi-card"
          onClick={() => onOpenWorkspace("modal-findings")}
          title="Click to open Findings workspace"
        >
          <div className="kpi-head">
            <span className="kpi-title">{kpi.securityFindings.title}</span>
            <svg
              className="kpi-icon"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            </svg>
          </div>
          <div className="kpi-value">{kpi.securityFindings.value}</div>
          <div className="kpi-sub">
            <span
              className="tag tag-high"
              style={{ fontSize: "9.5px", padding: "1px 5px" }}
            >
              {kpi.securityFindings.highCount ?? 0} HIGH
            </span>
            <span>{kpi.securityFindings.sub}</span>
          </div>
        </div>
      </div>

      {/* 2-Column Content Grid: Investigations Table & Findings */}
      <div className="dash-content-grid">
        {/* Recent Investigations Card */}
        <div className="dash-card">
          <div className="dash-card-header">
            <span className="dash-card-title">
              Recent Investigations &amp; Scripts
            </span>
            <button
              className="btn-small"
              onClick={() => onOpenWorkspace("modal-investigation")}
            >
              Open Script IDE →
            </button>
          </div>
          <div className="dash-card-body recent-investigations-scroll">
            <table className="dash-table">
              <thead>
                <tr>
                  <th>INVESTIGATION</th>
                  <th>TARGET FLEET</th>
                  <th>STATUS</th>
                  <th>LAST COMPILED</th>
                  <th>ACTION</th>
                </tr>
              </thead>
              <tbody>
                {investigations.map((inv, idx) => (
                  <tr key={inv.id || idx}>
                    <td>
                      <strong>{inv.title}</strong>
                    </td>
                    <td>{inv.target}</td>
                    <td>
                      <span
                        className={`tag ${inv.status === "READY" ? "tag-success" : "tag-info"}`}
                      >
                        {inv.status}
                      </span>
                    </td>
                    <td className="mono-small">{inv.lastCompiled}</td>
                    <td>
                      {idx === 0 ? (
                        <button
                          className="btn-view-details"
                          onClick={() => onOpenWorkspace("modal-investigation")}
                        >
                          Edit &amp; Run →
                        </button>
                      ) : idx === 1 ? (
                        <button
                          className="btn-view-details"
                          onClick={() => onOpenWorkspace("modal-evidence")}
                        >
                          View Evidence →
                        </button>
                      ) : (
                        <button
                          className="btn-view-details"
                          onClick={() => onOpenWorkspace("modal-findings")}
                        >
                          View Findings →
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* Recent Security Detections Card */}
        <div className="dash-card">
          <div className="dash-card-header">
            <span className="dash-card-title">Active Security Findings</span>
            <button
              className="btn-small"
              onClick={() => onOpenWorkspace("modal-findings")}
            >
              View All ({findings.length}) →
            </button>
          </div>
          <div className="dash-card-body">
            <div className="dash-findings-list">
              {findings.map((f, idx) => (
                <div className="dash-finding-item" key={f.id || idx}>
                  <div className="finding-left">
                    <div className="finding-title-row">
                      <span
                        className={`tag ${f.severity === "HIGH" ? "tag-high" : f.severity === "MEDIUM" ? "tag-medium" : "tag-low"}`}
                      >
                        {f.severity}
                      </span>
                      <span className="finding-title">{f.title}</span>
                    </div>
                    <span className="finding-meta">
                      Node: <strong>{f.machine}</strong> · {f.rationale}
                    </span>
                  </div>
                  <span
                    className="mono-small"
                    style={{ color: "var(--text-muted)", fontSize: "10.5px" }}
                  >
                    {f.time}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </main>
  );
}
