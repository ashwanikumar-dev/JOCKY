import React, { useState, useRef } from "react";
import { Modal } from "../components/Modal";
import { AlertModal } from "../components/AlertModal";
import { CodeEditor } from "../components/CodeEditor";
import { PipelineStatus } from "../components/PipelineStatus";
import { CompilerOutput } from "../components/CompilerOutput";
import {
  compileScript,
  createInvestigation,
  runInvestigation,
  getInvestigation,
  getEvidence,
  getFindings,
  getEvidenceItem,
  getTimeline,
} from "../services/api";

const DEFAULT_SCRIPT = `investigation "Endpoint Triage" {
    sys = System.info()
    processes = Process.collect()
    suspicious = processes.filter(name == "powershell.exe" and memory > 500MB)
    netconns = Network.connections()
    listeners = Network.listeners()
    ev = Evidence.preserve(suspicious)
    Evidence.verify(ev)
}`;

export function InvestigationModal({ isOpen, onClose }) {
  const [script, setScript] = useState(DEFAULT_SCRIPT);
  const [compileState, setCompileState] = useState("idle"); // "idle" | "compiling" | "success" | "error"
  const [pipelineStep, setPipelineStep] = useState("idle"); // "idle" | "parsing" | "ast" | "ir" | "complete" | "error"
  const [pipelineStatus, setPipelineStatus] = useState("IDLE"); // "IDLE" | "COMPILING" | "COMPLETE"
  const [compiledIR, setCompiledIR] = useState(null);
  const [ast, setAst] = useState(null);
  const [outputTab, setOutputTab] = useState("ir");
  const [headerStatus, setHeaderStatus] = useState("READY");
  const [lastCompileTime, setLastCompileTime] = useState("--");
  const [isReadyToRun, setIsReadyToRun] = useState(false);
  const [compilerSource, setCompilerSource] = useState(null);
  const editorTextareaRef = useRef(null);
  const [executionStatus, setExecutionStatus] = useState(null);
  const [findings, setFindings] = useState([]);
  const [evidenceItems, setEvidenceItems] = useState([]);
  const [currentTaskId, setCurrentTaskId] = useState(null);
  const [timelineEvents, setTimelineEvents] = useState([]);
  const [dispatchAlert, setDispatchAlert] = useState(null);

  const pollInvestigationStatus = async (investigationId, taskId) => {
    const poll = async () => {
      try {
        const response = await getInvestigation(investigationId);
        const investigation = response?.data || response;

        if (!investigation) {
          return;
        }

        const status =
          investigation.task_status || investigation.status || "unknown";

        setExecutionStatus(status);

        if (status === "completed") {
          try {
            const evidenceResponse = await getEvidence();

            const allEvidence = Array.isArray(evidenceResponse)
              ? evidenceResponse
              : [];

            const scopedEvidence = allEvidence.filter(
              (evidence) => String(evidence.task_id) === String(taskId),
            );

            setEvidenceItems(scopedEvidence);

            // Fetch all findings and scope them to this investigation/evidence.
            const findingsResponse = await getFindings(
              investigation.m5_investigation_id,
            );

            const allFindings = Array.isArray(findingsResponse?.data)
              ? findingsResponse.data
              : Array.isArray(findingsResponse)
                ? findingsResponse
                : [];

            const currentEvidenceIds = new Set(
              scopedEvidence.map((evidence) =>
                String(evidence.evidence_id ?? evidence.id),
              ),
            );

            const scopedFindings = allFindings.filter(
              (finding) =>
                String(finding.investigation_id) ===
                  String(investigation.m5_investigation_id) ||
                currentEvidenceIds.has(String(finding.evidence_id)),
            );

            setFindings(scopedFindings);
            const timelineResponse = await getTimeline(
              investigation.m5_investigation_id,
            );

            const investigationTimeline =
              timelineResponse?.data || timelineResponse || [];

            setTimelineEvents(
              Array.isArray(investigationTimeline) ? investigationTimeline : [],
            );
          } catch (error) {
            console.error("Failed to fetch investigation findings:", error);
          }

          return;
        }

        if (status === "failed") {
          return;
        }

        setTimeout(poll, 2000);
      } catch (error) {
        console.error("Failed to poll investigation status:", error);
      }
    };

    poll();
  };

  const handleCompile = async () => {
    // Duplicate click protection: ignore if compilation is already active
    if (compileState === "compiling") return;

    setCompileState("compiling");
    setPipelineStatus("COMPILING");
    setPipelineStep("parsing");
    setHeaderStatus("COMPILING...");
    setIsReadyToRun(false);

    // Natural compilation progression timers
    const t1 = setTimeout(() => {
      setPipelineStep((prev) => (prev === "parsing" ? "ast" : prev));
    }, 240);

    const t2 = setTimeout(() => {
      setPipelineStep((prev) => (prev === "ast" ? "ir" : prev));
    }, 480);

    // Minimum visual window (650ms) to ensure smooth forensic progression and avoid a jarring flash
    const minDelayPromise = new Promise((resolve) => setTimeout(resolve, 650));

    try {
      const [result] = await Promise.all([
        compileScript(script),
        minDelayPromise,
      ]);

      clearTimeout(t1);
      clearTimeout(t2);

      setCompiledIR(result.ir);
      setAst(result.ast);
      setCompilerSource(result.compiler_source);
      setPipelineStatus("COMPLETE");
      setPipelineStep("complete");
      setCompileState("success");

      const badgeText = result.compiler_source?.includes("REAL_RUST_COMPILER")
        ? "SCRIPT COMPILED (REAL RUST PARSER)"
        : "SCRIPT COMPILED";
      setHeaderStatus(badgeText);
      setIsReadyToRun(true);

      const now = new Date();
      setLastCompileTime(`${now.toUTCString().split(" ")[4]} UTC`);
    } catch (err) {
      clearTimeout(t1);
      clearTimeout(t2);

      setPipelineStatus("IDLE");
      setPipelineStep("error");
      setCompileState("error");
      setHeaderStatus("COMPILATION ERROR");
      setIsReadyToRun(false);
      setAst({ error: err.message, phase: "Lexer/Parser/Validation" });
      setCompiledIR({ error: err.message, status: "REJECTED" });
      setOutputTab("ast");
    }
  };

  const handleRunInvestigation = async () => {
    if (!isReadyToRun || !compiledIR) {
      return;
    }

    try {
      // 1. Persist the exact investigation that was compiled.
      const investigation = await createInvestigation({
        investigation_id: compiledIR.investigation_id,
        title: compiledIR.investigation_name || "Endpoint Triage",
        target: "Windows Fleet",
        script,
      });

      // 2. Dispatch the persisted investigation to a real M5 agent.
      const result = await runInvestigation(investigation.investigation_id);

      const taskId = result?.task?.task_id;
      const agentId = result?.agent_id;
      const machineId = result?.machine_id;

      setCurrentTaskId(taskId);

      setDispatchAlert({
        investigationId: investigation.investigation_id,
        taskId,
        agentId,
        machineId,
      });

      pollInvestigationStatus(investigation.investigation_id, taskId);

      setIsReadyToRun(false);
    } catch (error) {
      console.error("Failed to run investigation:", error);

      alert(
        `Investigation dispatch failed.\n\n${
          error?.message || "Unknown error"
        }`,
      );
    }
  };

  const handleSaveDraft = () => {
    alert("Investigation script draft saved successfully.");
  };

  const insertSnippet = (snippet) => {
    const textarea = editorTextareaRef.current;
    if (!textarea) {
      setScript((prev) => prev + "\n    " + snippet);
      return;
    }
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const before = script.substring(0, start);
    const after = script.substring(end);
    const newScript = before + "\n    " + snippet + after;
    setScript(newScript);
    setTimeout(() => {
      textarea.focus();
      textarea.selectionStart = textarea.selectionEnd =
        start + snippet.length + 5;
    }, 0);
  };

  const isCompleted = pipelineStatus === "COMPLETE";

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      id="modal-investigation"
      title="Investigation Workspace"
      icon={
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
        >
          <circle cx="11" cy="11" r="8" />
          <path d="m21 21-4.35-4.35" />
        </svg>
      }
      badge={
        <span
          id="inv-header-status"
          className={`tag ${
            executionStatus === "completed"
              ? "tag-success pulse-success"
              : executionStatus === "failed"
                ? "tag-error"
                : executionStatus === "created" ||
                    executionStatus === "running" ||
                    executionStatus === "dispatched"
                  ? "tag-medium"
                  : isCompleted
                    ? "tag-success pulse-success"
                    : compileState === "compiling"
                      ? "tag-medium"
                      : compileState === "error"
                        ? "tag-error"
                        : "tag-info"
          }`}
        >
          ●{" "}
          {executionStatus
            ? `EXECUTION ${executionStatus.toUpperCase()}`
            : headerStatus}
        </span>
      }
    >
      <div className="investigation-workspace-layout">
        {/* Top Area: Metadata & Actions */}
        <div className="inv-top-bar">
          <div className="inv-meta-block">
            <div className="inv-meta-title-row">
              <span className="inv-main-title">
                Endpoint Triage #IR-2026-0042
              </span>
              <span className="pill-label">Grammar v0.1</span>
            </div>
            <span className="inv-desc">
              Automated fleet-wide memory and connection inspection for
              suspicious PowerShell activity
            </span>
            <div className="inv-meta-sub">
              <span>
                Investigator: <strong>Investigator</strong>
              </span>
              <span>•</span>
              <span>
                Target: <strong>Windows Fleet (4 Nodes)</strong>
              </span>
              <span>•</span>
              <span>
                Created: <strong>23 Sep 2026</strong>
              </span>
            </div>
          </div>

          <div className="inv-top-actions">
            <button
              id="btn-save-script"
              className="btn-tertiary"
              title="Save draft script"
              onClick={handleSaveDraft}
            >
              <svg
                width="13"
                height="13"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
                <polyline points="17 21 17 13 7 13 7 21" />
                <polyline points="7 3 7 8 15 8" />
              </svg>
              <span>Save</span>
            </button>

            <button
              id="btn-compile"
              type="button"
              className={`btn-secondary ${compileState === "compiling" ? "compiling" : ""}`}
              onClick={handleCompile}
              disabled={compileState === "compiling"}
              aria-busy={compileState === "compiling"}
              aria-live="polite"
              title={
                compileState === "compiling"
                  ? "Compiling script..."
                  : "Compile Script (Ctrl+Enter)"
              }
            >
              {compileState === "compiling" ? (
                <>
                  <svg
                    className="compile-icon spinning"
                    width="13"
                    height="13"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2.2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    aria-hidden="true"
                  >
                    <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2" />
                  </svg>
                  <span>Compiling...</span>
                </>
              ) : (
                <>
                  <svg
                    className="compile-icon"
                    width="13"
                    height="13"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2.2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    aria-hidden="true"
                  >
                    <polyline points="16 18 22 12 16 6" />
                    <polyline points="8 6 2 12 8 18" />
                  </svg>
                  <span>Compile Script</span>
                </>
              )}
            </button>

            <button
              id="btn-submit-task"
              className={`btn-primary ${isReadyToRun && compileState !== "compiling" ? "ready-to-run" : ""}`}
              disabled={!isReadyToRun || compileState === "compiling"}
              aria-label="Run Investigation"
              onClick={handleRunInvestigation}
            >
              <span className="btn-run-icon-wrap">
                <svg
                  className="run-play-icon"
                  width="12"
                  height="12"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                >
                  <polygon points="6 4 20 12 6 20 6 4" />
                </svg>
              </span>
              <span className="btn-run-text">Run Investigation</span>
            </button>
          </div>
        </div>

        {/* Workflow Progress Header */}
        <div className="inv-workflow-bar">
          <div className="workflow-step completed" id="wf-step-1">
            <span className="step-circle">
              <svg
                width="10"
                height="10"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="3"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
            </span>
            <span>1. Write Script</span>
          </div>
          <div className="workflow-divider completed" />

          <div className="workflow-step completed" id="wf-step-2">
            <span className="step-circle">
              <svg
                width="10"
                height="10"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="3"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
            </span>
            <span>2. Select Machines</span>
          </div>
          <div
            className={`workflow-divider ${isCompleted ? "completed" : ""}`}
            id="wf-div-2"
          />

          <div
            className={`workflow-step ${isCompleted ? "completed" : "active"}`}
            id="wf-step-3"
          >
            <span className="step-circle">
              {isCompleted ? (
                <svg
                  width="10"
                  height="10"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="3"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <polyline points="20 6 9 17 4 12" />
                </svg>
              ) : (
                "3"
              )}
            </span>
            <span>3. Validate &amp; Compile</span>
          </div>
          <div
            className={`workflow-divider ${isCompleted ? "completed" : ""}`}
            id="wf-div-3"
          />

          <div
            className={`workflow-step ${isCompleted ? "completed" : ""}`}
            id="wf-step-4"
          >
            <span className="step-circle">
              {isCompleted ? (
                <svg
                  width="10"
                  height="10"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="3"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <polyline points="20 6 9 17 4 12" />
                </svg>
              ) : (
                "4"
              )}
            </span>
            <span>4. Review Plan</span>
          </div>
          <div className="workflow-divider" id="wf-div-4" />

          <div
            className={`workflow-step ${isCompleted ? "active" : ""}`}
            id="wf-step-5"
          >
            <span className="step-circle">5</span>
            <span>5. Execute</span>
          </div>
        </div>

        {/* Multi-Column Grid */}
        <div className="inv-main-grid">
          {/* Left Column: Code Editor & Validation/Output */}
          <div className="inv-left-col">
            <CodeEditor
              value={script}
              onChange={setScript}
              onCompile={handleCompile}
              editorRef={editorTextareaRef}
            />

            <div className="inv-compiler-row">
              <PipelineStatus
                status={pipelineStatus}
                step={pipelineStep}
                compileState={compileState}
              />
              <CompilerOutput
                compiledIR={compiledIR}
                ast={ast}
                activeTab={outputTab}
                onTabChange={setOutputTab}
              />
            </div>
          </div>

          {/* Right Column: JOCKY Reference (7 MVP Calls) & Targets */}
          <div className="inv-right-col">
            <div className="side-card">
              <div className="side-card-head">
                <span className="side-card-title">
                  JOCKY REFERENCE (7 MVP CALLS)
                </span>
                <span
                  className="pill-label"
                  style={{ fontSize: "9.5px", padding: "1px 5px" }}
                >
                  v0.1 Frozen
                </span>
              </div>
              <div className="side-card-body">
                <div className="ref-category">
                  <span className="ref-cat-label">System</span>
                  <div
                    className="ref-item"
                    onClick={() => insertSnippet("sys = System.info()")}
                  >
                    <span className="ref-item-name">System.info()</span>
                    <span className="ref-item-desc">
                      Snapshot host metadata, OS, architecture
                    </span>
                  </div>
                </div>

                <div className="ref-category">
                  <span className="ref-cat-label">Process</span>
                  <div
                    className="ref-item"
                    onClick={() => insertSnippet("procs = Process.collect()")}
                  >
                    <span className="ref-item-name">Process.collect()</span>
                    <span className="ref-item-desc">
                      Snapshot all running processes
                    </span>
                  </div>
                  <div
                    className="ref-item"
                    onClick={() =>
                      insertSnippet(
                        "filtered = processes.filter(memory > 500MB)",
                      )
                    }
                  >
                    <span className="ref-item-name">
                      Process.filter(predicate)
                    </span>
                    <span className="ref-item-desc">
                      Filter collected process list by condition
                    </span>
                  </div>
                </div>

                <div className="ref-category">
                  <span className="ref-cat-label">Network</span>
                  <div
                    className="ref-item"
                    onClick={() =>
                      insertSnippet("conns = Network.connections()")
                    }
                  >
                    <span className="ref-item-name">Network.connections()</span>
                    <span className="ref-item-desc">
                      Active TCP/UDP network connections
                    </span>
                  </div>
                  <div
                    className="ref-item"
                    onClick={() => insertSnippet("ports = Network.listeners()")}
                  >
                    <span className="ref-item-name">Network.listeners()</span>
                    <span className="ref-item-desc">
                      Open listening socket ports
                    </span>
                  </div>
                </div>

                <div className="ref-category">
                  <span className="ref-cat-label">Evidence</span>
                  <div
                    className="ref-item"
                    onClick={() =>
                      insertSnippet("ev = Evidence.preserve(target)")
                    }
                  >
                    <span className="ref-item-name">
                      Evidence.preserve(artifact)
                    </span>
                    <span className="ref-item-desc">
                      Cryptographic capture &amp; SHA-256 bundle
                    </span>
                  </div>
                  <div
                    className="ref-item"
                    onClick={() => insertSnippet("Evidence.verify(ev)")}
                  >
                    <span className="ref-item-name">
                      Evidence.verify(artifact)
                    </span>
                    <span className="ref-item-desc">
                      Verify cryptographic hash integrity
                    </span>
                  </div>
                </div>
              </div>
            </div>

            {/* Target Fleet Summary */}
            <div className="side-card">
              <div className="side-card-head">
                <span className="side-card-title">TARGET FLEET</span>
                <span className="tag tag-success">4 ONLINE</span>
              </div>
              <div className="side-card-body">
                <div
                  style={{
                    fontSize: "12px",
                    color: "var(--text-secondary)",
                    lineHeight: "1.6",
                  }}
                >
                  Targeting 4 enrolled endpoints in Windows/Linux Fleet.
                  Telemetry collection timeout: <strong>30s</strong>. Hash
                  verification: <strong>SHA-256</strong>.
                </div>
              </div>
            </div>
          </div>

          <div className="side-card findings-card">
            <div className="side-card-head">
              <span className="side-card-title">DETECTION FINDINGS</span>
              <span className="pill-label">{findings.length} FOUND</span>
            </div>

            <div className="side-card-body">
              {findings.length === 0 ? (
                <div
                  style={{
                    fontSize: "12px",
                    color: "var(--text-secondary)",
                  }}
                >
                  No findings detected.
                </div>
              ) : (
                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: "10px",
                  }}
                >
                  {findings.map((finding) => (
                    <div
                      key={finding.finding_id || finding.id}
                      style={{
                        padding: "10px",
                        border: "1px solid var(--border)",
                        borderRadius: "8px",
                        background: "var(--surface-secondary)",
                      }}
                    >
                      <div
                        style={{
                          display: "flex",
                          flexDirection: "column",
                          gap: "10px",
                          maxHeight: "300px",
                          overflowY: "auto",
                        }}
                      >
                        <strong style={{ fontSize: "12px" }}>
                          {finding.title || finding.rule_id}
                        </strong>

                        <span className="tag tag-error">
                          {finding.severity || "UNKNOWN"}
                        </span>
                      </div>

                      <div
                        style={{
                          marginTop: "6px",
                          fontSize: "11px",
                          color: "var(--text-secondary)",
                          lineHeight: "1.5",
                        }}
                      >
                        {finding.reason || "No reason provided."}
                      </div>

                      {finding.evidence_id && (
                        <div
                          style={{
                            marginTop: "6px",
                            fontSize: "10px",
                            color: "var(--text-muted)",
                          }}
                        >
                          Evidence #{finding.evidence_id}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
          <div className="side-card">
            <div className="side-card-head">
              <span className="side-card-title">PRESERVED EVIDENCE</span>

              <span className="pill-label">
                {evidenceItems.length} ARTIFACT
                {evidenceItems.length !== 1 ? "S" : ""}
              </span>
            </div>

            <div className="side-card-body">
              {evidenceItems.length === 0 ? (
                <div
                  style={{
                    fontSize: "12px",
                    color: "var(--text-secondary)",
                  }}
                >
                  No evidence collected yet.
                </div>
              ) : (
                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: "10px",
                    maxHeight: "300px",
                    overflowY: "auto",
                  }}
                >
                  {evidenceItems.map((evidence) => (
                    <div
                      key={evidence.evidence_id || evidence.id}
                      style={{
                        padding: "10px",
                        border: "1px solid var(--border)",
                        borderRadius: "8px",
                        background: "var(--surface-secondary)",
                      }}
                    >
                      <div
                        style={{
                          display: "flex",
                          justifyContent: "space-between",
                          alignItems: "center",
                          gap: "8px",
                        }}
                      >
                        <strong style={{ fontSize: "12px" }}>
                          {evidence.evidence_type ||
                            evidence.type ||
                            "Evidence"}
                        </strong>

                        <span className="tag tag-success">
                          {evidence.status || "VERIFIED"}
                        </span>
                      </div>

                      <div
                        style={{
                          marginTop: "7px",
                          fontSize: "11px",
                          color: "var(--text-secondary)",
                          lineHeight: "1.5",
                        }}
                      >
                        Evidence #{evidence.evidence_id || evidence.id}
                        <br />
                        Machine #{evidence.machine_id || "N/A"}
                        <br />
                        Collector version: {evidence.collector_version || "N/A"}
                        <br />
                        Collected:{" "}
                        {evidence.collectedAt ||
                          evidence.collection_time ||
                          "N/A"}
                      </div>

                      {evidence.sha256 && (
                        <div
                          style={{
                            marginTop: "8px",
                            fontSize: "9px",
                            color: "var(--text-muted)",
                            wordBreak: "break-all",
                          }}
                        >
                          SHA-256: {evidence.sha256}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
          <div className="side-card timeline-card">
            <div className="side-card-head">
              <span className="side-card-title">INVESTIGATION TIMELINE</span>
              <span className="pill-label">{timelineEvents.length} EVENTS</span>
            </div>

            <div className="timeline-scroll">
              {timelineEvents.length === 0 ? (
                <div
                  style={{
                    fontSize: "12px",
                    color: "var(--text-secondary)",
                  }}
                >
                  No timeline events recorded yet.
                </div>
              ) : (
                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: "10px",
                  }}
                >
                  {timelineEvents.map((event, index) => {
                    const isLast = index === timelineEvents.length - 1;

                    const eventType =
                      event.eventType || event.event_type || "EVENT";

                    const description = event.description || "Timeline event";

                    const evidenceId = event.evidenceRef || event.evidence_id;

                    const timestamp = event.timestamp || event.ts_utc;

                    const formattedTime = timestamp
                      ? new Date(timestamp).toLocaleTimeString([], {
                          hour: "2-digit",
                          minute: "2-digit",
                          second: "2-digit",
                        })
                      : "N/A";

                    return (
                      <div
                        key={event.timeline_id || event.id}
                        style={{
                          display: "flex",
                          minHeight: isLast ? "48px" : "58px",
                        }}
                      >
                        {/* Timeline rail */}
                        <div
                          style={{
                            width: "24px",
                            position: "relative",
                            display: "flex",
                            justifyContent: "center",
                            flexShrink: 0,
                          }}
                        >
                          {!isLast && (
                            <div
                              style={{
                                position: "absolute",
                                top: "16px",
                                bottom: "-1px",
                                width: "1px",
                                background: "var(--border)",
                              }}
                            />
                          )}

                          <div
                            style={{
                              width: "9px",
                              height: "9px",
                              borderRadius: "50%",
                              background: "var(--accent)",
                              border: "2px solid var(--surface)",
                              boxShadow: "0 0 0 1px var(--accent)",
                              marginTop: "7px",
                              zIndex: 1,
                            }}
                          />
                        </div>

                        {/* Event content */}
                        <div
                          style={{
                            flex: 1,
                            padding: "2px 0 10px 10px",
                            minWidth: 0,
                          }}
                        >
                          <div
                            style={{
                              display: "flex",
                              alignItems: "center",
                              justifyContent: "space-between",
                              gap: "12px",
                            }}
                          >
                            <span
                              style={{
                                fontSize: "12px",
                                fontWeight: 600,
                                color: "var(--text-primary)",
                              }}
                            >
                              {description}
                            </span>

                            <span
                              className="tag tag-info"
                              style={{
                                fontSize: "9px",
                                flexShrink: 0,
                              }}
                            >
                              {eventType}
                            </span>
                          </div>

                          <div
                            style={{
                              display: "flex",
                              alignItems: "center",
                              gap: "14px",
                              marginTop: "5px",
                              fontSize: "10px",
                              color: "var(--text-secondary)",
                            }}
                          >
                            {evidenceId && <span>Evidence #{evidenceId}</span>}

                            <span
                              style={{
                                color: "var(--text-muted)",
                              }}
                            >
                              {formattedTime} UTC
                            </span>
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Bottom Status Footer */}
        <div className="inv-status-footer">
          <div className="inv-footer-item">
            <span
              className="status-dot"
              style={{ background: "var(--green)" }}
            />
            <span>SYSTEM READY</span>
          </div>
          <div className="inv-footer-item">
            <span>
              FLEET: <strong>4 ENROLLED NODES</strong>
            </span>
          </div>
          <div className="inv-footer-item">
            <span>
              SUPPORTED OPERATIONS: <strong>7 MVP CALLS</strong>
            </span>
          </div>
          <div className="inv-footer-item" id="inv-last-compile">
            <span>
              Last Compile: <strong>{lastCompileTime}</strong>
            </span>
          </div>
        </div>
      </div>

      <AlertModal
        isOpen={!!dispatchAlert}
        onClose={() => setDispatchAlert(null)}
        type="success"
        title="Investigation Dispatched"
      >
        <div className="jocky-alert-details">
          <div className="jocky-alert-row jocky-alert-investigation">
            <span>Investigation</span>
            <strong>{dispatchAlert?.investigationId}</strong>
          </div>

          <div className="jocky-alert-row">
            <span>Task ID</span>
            <strong>#{dispatchAlert?.taskId ?? "N/A"}</strong>
          </div>

          <div className="jocky-alert-row">
            <span>Agent ID</span>
            <strong>#{dispatchAlert?.agentId ?? "N/A"}</strong>
          </div>

          <div className="jocky-alert-row">
            <span>Machine ID</span>
            <strong>#{dispatchAlert?.machineId ?? "N/A"}</strong>
          </div>

          <div className="jocky-alert-row">
            <span>Pipeline</span>
            <strong className="status-active">ACTIVE</strong>
          </div>

          <div className="jocky-alert-row">
            <span>Hash Verification</span>
            <strong className="status-verified">SHA-256 Enforced</strong>
          </div>
        </div>
      </AlertModal>
    </Modal>
  );
}
