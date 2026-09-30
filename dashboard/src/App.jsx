import React, { useState, useEffect, useCallback } from "react";
import { Navbar } from "./components/Navbar";
import { Subbar } from "./components/Subbar";
import { Footer } from "./components/Footer";

import { DashboardView } from "./pages/DashboardView";
import { InvestigationModal } from "./pages/InvestigationModal";
import { MachinesModal } from "./pages/MachinesModal";
import { MachineDetailModal } from "./pages/MachineDetailModal";
import { EvidenceModal } from "./pages/EvidenceModal";
import { FindingsModal } from "./pages/FindingsModal";
import { TimelineModal } from "./pages/TimelineModal";
import { AuditCustodyModal } from "./pages/AuditCustodyModal";
import { SettingsModal } from "./pages/SettingsModal";
import { Notification } from "./components/Notification";

import {
  getHealth,
  getDashboardStats,
  getMachines,
  getEvidence,
  getTimeline,
  getAuditLog,
} from "./services/api";

export default function App() {
  const [activeWorkspace, setActiveWorkspace] = useState(null);
  const [selectedMachineId, setSelectedMachineId] = useState("WIN-FORENSIC-01");
  const [searchValue, setSearchValue] = useState("");
  const [isBackendOnline, setIsBackendOnline] = useState(true);
  const [notification, setNotification] = useState(null);

  // Core Data States
  const [dashboardStats, setDashboardStats] = useState(null);
  const [machines, setMachines] = useState([]);
  const [evidence, setEvidence] = useState([]);
  const [findings, setFindings] = useState([]);
  const [timeline, setTimeline] = useState([]);
  const [auditLog, setAuditLog] = useState([]);

  const showNotification = useCallback((type, title, message = "") => {
    setNotification({
      type,
      title,
      message,
      id: Date.now(),
    });
  }, []);

  const dismissNotification = useCallback(() => {
    setNotification(null);
  }, []);

  // Load telemetry data from API
  const loadData = useCallback(async () => {
    try {
      // Check health
      await getHealth();
      setIsBackendOnline(true);
    } catch {
      setIsBackendOnline(false);
    }

    try {
      const statsRes = await getDashboardStats();
      const machRes = await getMachines();
      const evRes = await getEvidence();

      const latestInvestigation = statsRes?.recentInvestigations?.[0];

      const investigationId =
        latestInvestigation?.m5_investigation_id ||
        latestInvestigation?.id ||
        null;

      const timeRes = investigationId ? await getTimeline(investigationId) : [];

      const auditRes = investigationId
        ? await getAuditLog(investigationId)
        : [];

      setDashboardStats(statsRes);
      setMachines(machRes || []);
      setEvidence(evRes || []);
      setFindings(statsRes?.recentFindings || []);
      setTimeline(timeRes || []);
      setAuditLog(auditRes || []);
    } catch (err) {
      console.warn("Failed to fetch initial telemetry data:", err);
    }
  }, []);

  useEffect(() => {
    loadData();
    // Poll telemetry periodically every 30 seconds
    const interval = setInterval(loadData, 30000);
    return () => clearInterval(interval);
  }, [loadData]);

  // Handle URL hash deep linking (matching original app.js)
  useEffect(() => {
    if (window.location.hash) {
      const parts = window.location.hash.substring(1).split("&");
      const hashModal = parts[0];
      let machineTarget = "WIN-FORENSIC-01";
      parts.forEach((p) => {
        if (p.startsWith("target=")) machineTarget = p.split("=")[1];
      });

      if (hashModal === "modal-machine-detail") {
        setSelectedMachineId(machineTarget);
        setActiveWorkspace("modal-machine-detail");
      } else if (hashModal === "modal-reports") {
        setActiveWorkspace("modal-audit-custody");
      } else if (
        [
          "modal-investigation",
          "modal-machines",
          "modal-evidence",
          "modal-findings",
          "modal-timeline",
          "modal-audit-custody",
          "modal-settings",
        ].includes(hashModal)
      ) {
        setActiveWorkspace(hashModal);
      }
    }
  }, []);

  // Keyboard navigation shortcuts (matching original app.js)
  useEffect(() => {
    const handleKeyDown = (e) => {
      const isTyping = ["TEXTAREA", "INPUT", "SELECT"].includes(
        document.activeElement?.tagName,
      );

      if (e.key === "Escape") {
        if (activeWorkspace === "modal-machine-detail") {
          setActiveWorkspace("modal-machines");
        } else {
          setActiveWorkspace(null);
        }
      } else if (!isTyping && !e.ctrlKey && !e.metaKey && !e.altKey) {
        if (e.key === "d" || e.key === "D" || e.key === "0") {
          setActiveWorkspace(null);
          window.scrollTo({ top: 0, behavior: "smooth" });
        } else {
          const keyMap = {
            1: "modal-investigation",
            2: "modal-machines",
            3: "modal-evidence",
            4: "modal-findings",
            5: "modal-timeline",
            6: "modal-audit-custody",
          };
          if (keyMap[e.key]) {
            setActiveWorkspace(keyMap[e.key]);
          }
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [activeWorkspace]);

  // Selected machine object
  const currentMachine =
    machines.find(
      (m) =>
        m.id === selectedMachineId ||
        m.hostname?.toLowerCase() === selectedMachineId?.toLowerCase(),
    ) || machines[0];

  const handleOpenMachineDetail = (machineId) => {
    setSelectedMachineId(machineId);
    setActiveWorkspace("modal-machine-detail");
  };

  const handleStartInvestigationForMachine = () => {
    setActiveWorkspace("modal-investigation");
  };

  return (
    <>
      {/* 1. Header / Navbar */}
      <Navbar
        searchValue={searchValue}
        onSearchChange={setSearchValue}
        onSearchFocus={() => {}}
        nodeCount={machines.length || 4}
        isOnline={isBackendOnline}
        onLogoClick={() => setActiveWorkspace(null)}
      />

      {/* 2. Subbar with Workspaces Menu */}
      <Subbar
        activeWorkspace={activeWorkspace}
        onSelectWorkspace={setActiveWorkspace}
        activeCaseTitle={dashboardStats?.systemStatus?.activeCase}
      />

      {/* 3. Main Dashboard View */}
      <DashboardView
        stats={dashboardStats}
        investigations={dashboardStats?.recentInvestigations || []}
        findings={findings}
        onOpenWorkspace={setActiveWorkspace}
        onStartInvestigation={handleStartInvestigationForMachine}
      />

      {/* 4. Footer */}
      <Footer nodeCount={machines.length || 4} />

      {/* ════ WORKSPACE MODALS ════ */}

      {/* 1. Investigation Modal */}
      <InvestigationModal
        isOpen={activeWorkspace === "modal-investigation"}
        onClose={() => setActiveWorkspace(null)}
        showNotification={showNotification}
      />

      {/* 2. Machines Modal */}
      <MachinesModal
        isOpen={activeWorkspace === "modal-machines"}
        onClose={() => setActiveWorkspace(null)}
        machines={machines}
        onSelectMachine={handleOpenMachineDetail}
        onRefresh={loadData}
      />

      {/* 3. Machine Detail Modal */}
      <MachineDetailModal
        isOpen={activeWorkspace === "modal-machine-detail"}
        onClose={() => setActiveWorkspace(null)}
        machine={currentMachine}
        onBackToMachines={() => setActiveWorkspace("modal-machines")}
        onStartInvestigation={handleStartInvestigationForMachine}
        onViewEvidence={() => setActiveWorkspace("modal-evidence")}
        onViewFindings={() => setActiveWorkspace("modal-findings")}
      />

      {/* 4. Evidence Modal */}
      <EvidenceModal
        isOpen={activeWorkspace === "modal-evidence"}
        onClose={() => setActiveWorkspace(null)}
        evidence={evidence}
      />

      {/* 5. Findings Modal */}
      <FindingsModal
        isOpen={activeWorkspace === "modal-findings"}
        onClose={() => setActiveWorkspace(null)}
        findings={findings}
      />

      {/* 6. Timeline Modal */}
      <TimelineModal
        isOpen={activeWorkspace === "modal-timeline"}
        onClose={() => setActiveWorkspace(null)}
        timeline={timeline}
      />

      {/* 7. Reports / Audit-Custody Modal */}
      <AuditCustodyModal
        isOpen={activeWorkspace === "modal-audit-custody"}
        onClose={() => setActiveWorkspace(null)}
        auditLog={auditLog}
      />

      {/* 8. Settings Modal */}
      <SettingsModal
        isOpen={activeWorkspace === "modal-settings"}
        onClose={() => setActiveWorkspace(null)}
        isOnline={isBackendOnline}
      />

      <Notification notification={notification} onClose={dismissNotification} />
    </>
  );
}
