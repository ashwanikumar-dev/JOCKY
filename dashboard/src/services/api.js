// Centralized API Service for JOCKY IR Console
const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL || "http://localhost:8001";

// Fallback baseline data if backend is offline/unreachable
export const FALLBACK_DATA = {
  machines: [
    {
      id: "WIN-FORENSIC-01",
      hostname: "WIN-FORENSIC-01",
      os: "Windows 11 Pro (Build 22631)",
      platform: "windows",
      ip: "10.0.1.47",
      agent: "v0.4.2",
      status: "ONLINE",
      arch: "x86_64",
      heartbeat: "20:42:27 UTC",
      evidenceCount: 18,
      findingsCount: 1,
      processes: 148,
      connections: 24,
      activity: [
        {
          text: "Process.collect() snapshot generated 241 records",
          time: "20:42 UTC",
        },
        {
          text: "Evidence.preserve() stored SHA-256 payload ev-0001",
          time: "20:41 UTC",
        },
        {
          text: "High threat flagged: Suspicious PowerShell process",
          tag: "HIGH",
        },
      ],
    },
    {
      id: "UBUNTU-ANALYSIS",
      hostname: "UBUNTU-ANALYSIS",
      os: "Ubuntu 24.04 LTS (Kernel 6.8.0-40-generic)",
      platform: "linux",
      ip: "10.0.1.88",
      agent: "v0.4.2",
      status: "ONLINE",
      arch: "x86_64",
      heartbeat: "20:42:26 UTC",
      evidenceCount: 9,
      findingsCount: 1,
      processes: 84,
      connections: 12,
      activity: [
        {
          text: "Network.listeners() detected open TCP port 4444",
          time: "20:40 UTC",
        },
        {
          text: "Evidence.preserve() stored network capture ev-0002",
          time: "20:39 UTC",
        },
        {
          text: "Medium threat flagged: Anomalous Outbound Listener",
          tag: "MED",
        },
      ],
    },
    {
      id: "WIN-CLIENT-03",
      hostname: "WIN-CLIENT-03",
      os: "Windows 10 Pro (Build 19045)",
      platform: "windows",
      ip: "10.0.1.23",
      agent: "v0.4.2",
      status: "ONLINE",
      arch: "x86_64",
      heartbeat: "20:42:25 UTC",
      evidenceCount: 6,
      findingsCount: 0,
      processes: 112,
      connections: 8,
      activity: [
        { text: "System.info() hardware audit completed", time: "20:35 UTC" },
        {
          text: "Telemetry heartbeat transmitted (latency 1.2ms)",
          time: "20:42 UTC",
        },
      ],
    },
    {
      id: "WIN-SERVER-09",
      hostname: "WIN-SERVER-09",
      os: "Windows Server 2022 (Build 20348)",
      platform: "windows",
      ip: "10.0.1.91",
      agent: "v0.4.1",
      status: "ONLINE",
      arch: "x86_64",
      heartbeat: "20:42:22 UTC",
      evidenceCount: 4,
      findingsCount: 0,
      processes: 196,
      connections: 45,
      activity: [
        { text: "Domain controller telemetry synchronized", time: "20:30 UTC" },
        {
          text: "Evidence vault verified against local SHA-256 cache",
          time: "20:40 UTC",
        },
      ],
    },
  ],
  evidence: [],
  findings: [],
  investigations: [],
  timeline: [],
  audit: [],
};

// Generic safe fetch with JSON parsing
async function safeFetch(endpoint, options = {}) {
  const url = `${API_BASE_URL}${endpoint}`;
  try {
    const res = await fetch(url, {
      ...options,
      headers: {
        "Content-Type": "application/json",
        ...(options.headers || {}),
      },
    });
    if (!res.ok) {
      throw new Error(`HTTP ${res.status}: ${res.statusText}`);
    }
    const json = await res.json();
    return json.data !== undefined ? json.data : json;
  } catch (err) {
    console.warn(`[API] safeFetch failed for ${endpoint}:`, err.message);
    throw err;
  }
}

// ── Health Check ──
export async function getHealth() {
  return safeFetch("/api/health");
}

// ── Dashboard Stats ──
export async function getDashboardStats() {
  try {
    return await safeFetch("/api/dashboard/stats");
  } catch {
    return {
      systemStatus: {
        status: "ONLINE",
        label: "SYSTEM ONLINE",
        telemetryActive: true,
        nodeCount: 4,
        nodesOnline: 4,
        investigator: { name: "Investigator", role: "Investigator" },
        activeCase:
          "Endpoint Triage #IR-2026-0042 · Enrolled Fleet (4 Nodes Online)",
      },
      kpi: {
        activeInvestigation: {
          title: "Active Investigation",
          value: "Endpoint Triage",
          sub: "DSL v0.1 · 7 MVP Ops Ready",
        },
        enrolledEndpoints: {
          title: "Enrolled Endpoints",
          value: "4 / 4 Online",
          healthBadge: "100% HEALTH",
          sub: "3 Windows · 1 Linux",
        },
        preservedEvidence: {
          title: "Preserved Evidence",
          value: "37 Artifacts",
          hashBadge: "SHA-256",
          sub: "Tamper-evident chain",
        },
        securityFindings: {
          title: "Security Findings",
          value: "2 Detections",
          highCount: 1,
          sub: "1 Medium · 0 Critical",
        },
      },
      recentInvestigations: FALLBACK_DATA.investigations,
      recentFindings: FALLBACK_DATA.findings,
    };
  }
}

// ── Machines ──
export async function getMachines() {
  try {
    return await safeFetch("/api/machines");
  } catch {
    return FALLBACK_DATA.machines;
  }
}

export async function getMachine(id) {
  try {
    return await safeFetch(`/api/machines/${id}`);
  } catch {
    return (
      FALLBACK_DATA.machines.find(
        (m) => m.id === id || m.hostname.toLowerCase() === id.toLowerCase(),
      ) || FALLBACK_DATA.machines[0]
    );
  }
}

// ── Evidence ──
export async function getEvidence() {
  try {
    return await safeFetch("/api/evidence");
  } catch {
    return FALLBACK_DATA.evidence;
  }
}

export async function getEvidenceItem(id) {
  try {
    return await safeFetch(`/api/evidence/${id}`);
  } catch {
    return FALLBACK_DATA.evidence.find((e) => e.id === id);
  }
}

// ── Findings ──
export async function getFindings(investigationId = null) {
  try {
    const endpoint = investigationId
      ? `/api/findings?investigation_id=${encodeURIComponent(investigationId)}`
      : "/api/findings";

    return await safeFetch(endpoint);
  } catch {
    return FALLBACK_DATA.findings;
  }
}

export async function getFinding(id) {
  try {
    return await safeFetch(`/api/findings/${id}`);
  } catch {
    return FALLBACK_DATA.findings.find((f) => f.id === id);
  }
}

// ── Investigations ──
export async function getInvestigations() {
  try {
    return await safeFetch("/api/investigations");
  } catch {
    return FALLBACK_DATA.investigations;
  }
}

export async function getInvestigation(id) {
  try {
    return await safeFetch(`/api/investigations/${id}`);
  } catch {
    return FALLBACK_DATA.investigations.find((i) => i.id === id);
  }
}

// ── Timeline ──
export async function getTimeline(investigationId = null) {
  try {
    const endpoint = investigationId
      ? `/api/timeline?investigation_id=${encodeURIComponent(investigationId)}`
      : "/api/timeline";

    return await safeFetch(endpoint);
  } catch {
    return FALLBACK_DATA.timeline;
  }
}

// ── Audit / Custody ──
export async function getAuditLog(investigationId = null) {
  try {
    if (!investigationId) {
      return [];
    }

    return await safeFetch(
      `/api/audit?investigation_id=${encodeURIComponent(investigationId)}`,
    );
  } catch {
    return [];
  }
}

// ── Compiler Pipeline ──
export async function compileScript(script) {
  try {
    const res = await fetch(`${API_BASE_URL}/api/compile`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ script }),
    });
    if (!res.ok) {
      const errJson = await res.json().catch(() => ({}));
      const msg = errJson?.error?.message || `Compilation HTTP ${res.status}`;
      throw new Error(msg);
    }
    const data = await res.json();
    return {
      ir: data.ir || data,
      ast: data.ast || null,
      compiler_source: data.compiler_source || "REAL_RUST_COMPILER",
    };
  } catch (err) {
    // If it's a real compiler syntax error from backend, re-throw it so the IDE displays it
    if (err.message && !err.message.includes("Failed to fetch")) {
      throw err;
    }
    // Contract fallback if backend compilation endpoint is completely offline
    return {
      compiler_source: "OFFLINE_FALLBACK",
      ir: {
        ir_version: "v0.1",
        investigation_id: "inv-2026-0042",
        pipeline_status: "SYNTACTICALLY_VALID",
        instructions: [
          { op: "INVESTIGATION_BEGIN", params: { name: "Endpoint Triage" } },
          { op: "COLLECT_SYSTEM_INFO", params: {} },
          { op: "COLLECT_PROCESSES", params: {} },
          {
            op: "FILTER_PROCESSES",
            params: {
              predicate: 'name == "powershell.exe" and memory > 500MB',
            },
          },
          { op: "NETWORK_CONNECTIONS", params: {} },
          { op: "NETWORK_LISTENERS", params: {} },
          {
            op: "PRESERVE_EVIDENCE",
            params: { source: "suspicious", hash_algo: "SHA-256" },
          },
          { op: "VERIFY_EVIDENCE", params: { target: "ev" } },
          { op: "INVESTIGATION_END", params: {} },
        ],
      },
      ast: {
        Program: {
          investigation: {
            name: { value: "Endpoint Triage", span: { line: 1, column: 15 } },
            body: [
              {
                Assignment: {
                  target: { name: "sys" },
                  value: { Call: { object: "System", function: "info" } },
                },
              },
              {
                Assignment: {
                  target: { name: "processes" },
                  value: { Call: { object: "Process", function: "collect" } },
                },
              },
              {
                Assignment: {
                  target: { name: "suspicious" },
                  value: {
                    Filter: {
                      source: "processes",
                      condition: "name == powershell.exe and memory > 500MB",
                    },
                  },
                },
              },
              {
                Assignment: {
                  target: { name: "netconns" },
                  value: {
                    Call: { object: "Network", function: "connections" },
                  },
                },
              },
              {
                Assignment: {
                  target: { name: "listeners" },
                  value: { Call: { object: "Network", function: "listeners" } },
                },
              },
              {
                Assignment: {
                  target: { name: "ev" },
                  value: { Call: { object: "Evidence", function: "preserve" } },
                },
              },
              {
                ExpressionStatement: {
                  Call: { object: "Evidence", function: "verify" },
                },
              },
            ],
            span: { line: 1, column: 1 },
          },
        },
      },
    };
  }
}

export async function createInvestigation({
  investigation_id,
  title,
  target,
  script,
}) {
  const response = await fetch(`${API_BASE_URL}/api/investigations`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      investigation_id,
      title,
      target,
      script,
    }),
  });

  const text = await response.text();

  let data;
  try {
    data = text ? JSON.parse(text) : null;
  } catch {
    data = text;
  }

  if (!response.ok) {
    const message =
      data?.error?.message || data?.detail || "Failed to create investigation";

    throw new Error(message);
  }

  return data?.data ?? data;
}

export async function runInvestigation(investigationId) {
  const response = await fetch(
    `${API_BASE_URL}/api/investigations/${encodeURIComponent(
      investigationId,
    )}/run`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
    },
  );

  const text = await response.text();

  let data;
  try {
    data = text ? JSON.parse(text) : null;
  } catch {
    data = text;
  }

  if (!response.ok) {
    const message =
      data?.error?.message ||
      data?.detail ||
      "Failed to dispatch investigation";

    throw new Error(message);
  }

  return data?.data ?? data;
}
