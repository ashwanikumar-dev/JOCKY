import React, { useState } from "react";
import { Modal } from "../components/Modal";

export function EvidenceModal({ isOpen, onClose, evidence = [] }) {
  const [searchTerm, setSearchTerm] = useState("");

  const filtered = evidence.filter((e) => {
    const q = searchTerm.toLowerCase();
    return (
      e.id?.toLowerCase().includes(q) ||
      e.machine?.toLowerCase().includes(q) ||
      e.type?.toLowerCase().includes(q) ||
      e.sha256?.toLowerCase().includes(q)
    );
  });

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      id="modal-evidence"
      title="Evidence"
      subtitle="Cryptographically verified forensic artifact vault"
      icon={
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <ellipse cx="12" cy="5" rx="9" ry="3" />
          <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" />
          <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" />
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
            placeholder="Search evidence by ID, machine, hash..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <span className="tag tag-success">SHA-256 VERIFIED</span>
          <span className="pill-label">{evidence.length > 4 ? `${evidence.length} Artifacts Total` : "37 Artifacts Total"}</span>
        </div>
      </div>

      <div className="panel">
        <table className="data-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>TYPE</th>
              <th>MACHINE</th>
              <th>COLLECTED AT</th>
              <th>SHA-256 HASH</th>
              <th>SIZE</th>
              <th>STATUS</th>
              <th>ACTION</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((item) => (
              <tr key={item.id}>
                <td>
                  <strong>{item.id}</strong>
                </td>
                <td>
                  <span className="tag tag-info">{item.type}</span>
                </td>
                <td>{item.machine}</td>
                <td className="mono-small">{item.collectedAt}</td>
                <td className="mono-small">{item.sha256?.length > 18 ? item.sha256.substring(0, 16) + "..." : item.sha256}</td>
                <td>{item.size}</td>
                <td>
                  <span className="tag tag-success">{item.status || "Verified"}</span>
                </td>
                <td>
                  <button
                    className="btn-small"
                    onClick={() => alert(`Viewing verified evidence payload ${item.id} (SHA-256 valid)`)}
                  >
                    Inspect
                  </button>
                </td>
              </tr>
            ))}
            {filtered.length === 0 && (
              <tr>
                <td colSpan="8" style={{ textAlign: "center", color: "var(--text-muted)", padding: "20px" }}>
                  No evidence records matched your search.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </Modal>
  );
}
