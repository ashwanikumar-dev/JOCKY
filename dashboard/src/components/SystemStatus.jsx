import React, { useState, useEffect } from "react";

export function SystemStatus({
  nodeCount = 4,
  investigator = { name: "Investigator", role: "Investigator" },
  isOnline = true,
}) {
  const [utcTime, setUtcTime] = useState("-- UTC");

  useEffect(() => {
    const updateTime = () => {
      const now = new Date();
      const d = String(now.getUTCDate()).padStart(2, "0");
      const months = [
        "Jan",
        "Feb",
        "Mar",
        "Apr",
        "May",
        "Jun",
        "Jul",
        "Aug",
        "Sep",
        "Oct",
        "Nov",
        "Dec",
      ];
      const m = months[now.getUTCMonth()];
      const y = now.getUTCFullYear();
      const t = now.toUTCString().split(" ")[4];

      setUtcTime(`${d} ${m} ${y} ${t} UTC`);
    };

    updateTime();
    const interval = setInterval(updateTime, 1000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="topbar-right">
      <div
        className={`system-status ${isOnline ? "online" : "offline"}`}
        title={isOnline ? "Backend connected" : "Backend service unavailable"}
      >
        <span className="status-dot pulse" />
        <span className="status-label">
          {isOnline ? "ONLINE" : "BACKEND OFFLINE"}
        </span>
        <span id="live-utc-clock" className="status-time">
          {utcTime}
        </span>
      </div>

      <div className="node-badge" title="Active Fleet Nodes">
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
        >
          <rect x="2" y="3" width="20" height="14" rx="2" />
          <path d="M8 21h8M12 17v4" />
        </svg>
        <span>{nodeCount} Nodes</span>
      </div>

      <div className="user-avatar" title="Investigator Profile">
        <div className="avatar-circle">
          {investigator.name ? investigator.name[0] : "I"}
        </div>

        <div className="user-info">
          <span className="user-name">
            {investigator.name || "Investigator"}
          </span>

          <span className="user-role">
            {investigator.role || "Investigator"}
          </span>
        </div>
      </div>
    </div>
  );
}
