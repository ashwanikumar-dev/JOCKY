import React from "react";
import { Modal } from "../components/Modal";

export function TimelineModal({ isOpen, onClose, timeline = [] }) {
  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      id="modal-timeline"
      title="Timeline"
      subtitle="Correlated chronological forensic event stream across all enrolled endpoints"
      icon={
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <circle cx="12" cy="12" r="10" />
          <polyline points="12 6 12 12 16 14" />
        </svg>
      }
    >
      <div className="panel">
        <table className="data-table">
          <thead>
            <tr>
              <th>TIMESTAMP (UTC)</th>
              <th>SOURCE NODE</th>
              <th>EVENT TYPE</th>
              <th>DESCRIPTION</th>
              <th>EVIDENCE REF</th>
            </tr>
          </thead>
          <tbody>
            {timeline.map((evt, idx) => (
              <tr key={evt.id || idx}>
                <td className="mono-small">{evt.timestamp}</td>
                <td>{evt.sourceNode}</td>
                <td>
                  <span className={`tag ${evt.eventType === "NETWORK" ? "tag-high" : "tag-info"}`}>
                    {evt.eventType}
                  </span>
                </td>
                <td>{evt.description}</td>
                <td>
                  <span className="mono-small">{evt.evidenceRef}</span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </Modal>
  );
}
