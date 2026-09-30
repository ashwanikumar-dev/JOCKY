import React, { useState } from "react";
import { Modal } from "../components/Modal";

export function FindingsModal({ isOpen, onClose, findings = [] }) {
  const [searchTerm, setSearchTerm] = useState("");

  const filtered = findings.filter((f) => {
    const q = searchTerm.toLowerCase();
    return (
      f.title?.toLowerCase().includes(q) ||
      f.machine?.toLowerCase().includes(q) ||
      f.severity?.toLowerCase().includes(q) ||
      f.rationale?.toLowerCase().includes(q)
    );
  });

  const highCount = findings.filter((f) => f.severity === "HIGH").length;
  const medCount = findings.filter((f) => f.severity === "MEDIUM").length;

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      id="modal-findings"
      title="Findings"
      subtitle="Correlated detection alerts and threat indicators"
      icon={
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
        </svg>
      }
    >
      <div className="machines-toolbar">
        <div className="machines-search-wrap">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="11" cy="11" r="8" />
            <path d="m21 21-4.35-4.35" />
          </svg>
          <input
            type="text"
            placeholder="Search findings by title, node, rule..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <span className="tag tag-high">{highCount} HIGH</span>
          <span className="tag tag-medium">{medCount} MEDIUM</span>
        </div>
      </div>

      <div className="panel">
        <table className="data-table">
          <thead>
            <tr>
              <th>SEVERITY</th>
              <th>FINDING</th>
              <th>MACHINE</th>
              <th>RATIONALE</th>
              <th>TIME</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((item) => (
              <tr key={item.id}>
                <td>
                  <span className={`tag ${item.severity === "HIGH" ? "tag-high" : item.severity === "MEDIUM" ? "tag-medium" : "tag-low"}`}>
                    {item.severity}
                  </span>
                </td>
                <td>
                  <strong>{item.title}</strong>
                </td>
                <td>{item.machine}</td>
                <td>{item.rationale}</td>
                <td className="mono-small">{item.time}</td>
              </tr>
            ))}
            {filtered.length === 0 && (
              <tr>
                <td colSpan="5" style={{ textAlign: "center", color: "var(--text-muted)", padding: "20px" }}>
                  No findings matched your search.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </Modal>
  );
}
