import React, { useState } from "react";
import { Modal } from "../components/Modal";
import { MachineCard } from "../components/MachineCard";

export function MachinesModal({ isOpen, onClose, machines = [], onSelectMachine, onRefresh }) {
  const [searchTerm, setSearchTerm] = useState("");

  const filteredMachines = machines.filter((m) => {
    const q = searchTerm.toLowerCase();
    return (
      m.hostname?.toLowerCase().includes(q) ||
      m.os?.toLowerCase().includes(q) ||
      m.ip?.toLowerCase().includes(q)
    );
  });

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      id="modal-machines"
      title="Machines"
      subtitle="Manage and query your endpoints"
      icon={
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <rect x="2" y="3" width="20" height="14" rx="2" />
          <path d="M8 21h8M12 17v4" />
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
            id="machine-search-input"
            placeholder="Search machines by hostname, OS, IP..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <span className="pill-label">{machines.length} Endpoints Connected</span>
          <button
            className="btn-small"
            onClick={() => {
              if (onRefresh) onRefresh();
              alert(`Telemetry refreshed across all ${machines.length} fleet nodes.`);
            }}
            style={{ display: "inline-flex", alignItems: "center", gap: "5px" }}
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M23 4v6h-6" />
              <path d="M1 20v-6h6" />
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
            </svg>
            <span>Refresh</span>
          </button>
        </div>
      </div>

      <div className="machines-grid" id="machines-cards-grid">
        {filteredMachines.map((machine) => (
          <MachineCard
            key={machine.id || machine.hostname}
            machine={machine}
            onSelect={onSelectMachine}
          />
        ))}
        {filteredMachines.length === 0 && (
          <div style={{ padding: "30px", textAlign: "center", color: "var(--text-muted)", gridColumn: "1 / -1" }}>
            No matching machines found.
          </div>
        )}
      </div>
    </Modal>
  );
}
