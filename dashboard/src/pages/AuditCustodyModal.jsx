import React from "react";
import { Modal } from "../components/Modal";

export function AuditCustodyModal({ isOpen, onClose, auditLog = [] }) {
  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      id="modal-audit-custody"
      title="Audit/Custody"
      subtitle="Unified cryptographic chain of custody and immutable audit log"
      icon={
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
          <polyline points="14 2 14 8 20 8" />
          <line x1="16" y1="13" x2="8" y2="13" />
          <line x1="16" y1="17" x2="8" y2="17" />
          <polyline points="10 9 9 9 8 9" />
        </svg>
      }
    >
      <div className="panel">
        <table className="data-table">
          <thead>
            <tr>
              <th>ENTRY ID</th>
              <th>ACTION</th>
              <th>OPERATOR</th>
              <th>TIMESTAMP (UTC)</th>
              <th>PREVIOUS BLOCK HASH</th>
              <th>CURRENT RECORD HASH</th>
              <th>VERIFICATION</th>
            </tr>
          </thead>
          <tbody>
            {auditLog.map((log, idx) => (
              <tr key={log.entryId || idx}>
                <td>
                  <strong>{log.entryId}</strong>
                </td>
                <td>{log.action}</td>
                <td>{log.operator}</td>
                <td className="mono-small">{log.timestamp}</td>
                <td className="mono-small">{log.prevHash?.length > 18 ? log.prevHash.substring(0, 14) + "..." : log.prevHash}</td>
                <td className="mono-small">{log.currentHash?.length > 18 ? log.currentHash.substring(0, 14) + "..." : log.currentHash}</td>
                <td>
                  <span className="tag tag-success">{log.verification || "VERIFIED"}</span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </Modal>
  );
}
