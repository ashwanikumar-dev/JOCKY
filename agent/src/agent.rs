use crate::collector::Collector;
use crate::config::{AgentConfig, AgentState};
use crate::evidence::{AgentTask, Evidence, EvidenceStatus, RawEvidence, TaskStatusUpdate};
use crate::runtime::Runtime;
use crate::windows_collectors::{
    network::WindowsNetworkCollector, process::WindowsProcessCollector, system::WindowsSystemCollector,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct RegReq { machine_id: String, hostname: String, os: String, agent_version: String }

#[derive(Debug, serde::Deserialize)]
struct RegRes {
    agent_id: String,
    machine_id: String,
}

pub struct JockyAgent {
    pub config: AgentConfig,
    pub state: AgentState,
    client: reqwest::Client,
    pub system: WindowsSystemCollector,
    #[allow(dead_code)]
    pub process: WindowsProcessCollector,
    #[allow(dead_code)]
    pub network: WindowsNetworkCollector,
}

impl JockyAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            state: AgentState::load(&config.state_file),
            config,
            client: reqwest::Client::new(),
            system: WindowsSystemCollector::new(),
            process: WindowsProcessCollector::new(),
            network: WindowsNetworkCollector::new(),
        }
    }

    pub fn collect_system_info(&self, params: &Value) -> Result<RawEvidence, crate::collector::CollectorError> {
        self.system.collect(params)
    }
    pub fn collect_processes(
    &self,
    params: &Value,
) -> Result<RawEvidence, crate::collector::CollectorError> {
    self.process.collect(params)
}

pub fn collect_network(
    &self,
    params: &Value,
) -> Result<RawEvidence, crate::collector::CollectorError> {
    self.network.collect(params)
}

    #[inline]
    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.config.backend_url.trim_end_matches('/'), path.trim_start_matches('/'))
    }
    
    pub async fn register(&mut self) -> Result<String> {
        if let Some(ref id) = self.state.agent_id { return Ok(id.clone()); }

        let req = RegReq {
            machine_id: self.config.machine_id.clone(),
            hostname: self.config.machine_id.clone(),
            os: "windows".into(),
            agent_version: env!("CARGO_PKG_VERSION").into(),
        };

        let res = self.client.post(self.url("agents/register")).json(&req).send().await?;
        if !res.status().is_success() { bail!("Registration rejected: HTTP {}", res.status()); }

        let reg = res.json::<RegRes>().await?;

let id = reg.agent_id;
let machine_id = reg.machine_id;

self.state.agent_id = Some(id.clone());
self.state.machine_id = Some(machine_id.clone());

let _ = self.state.save(&self.config.state_file);

info!("Assigned agent_id: {}", id);
info!("Assigned backend machine_id: {}", machine_id);

Ok(id)
    }

    pub async fn poll_tasks(&self) -> Result<Option<AgentTask>> {
        info!("POLL: starting task poll");
    let id = self
        .state
        .agent_id
        .as_deref()
        .context("Unregistered")?;

    let res = self
        .client
        .get(self.url(&format!("tasks/poll?agent_id={}", id)))
        .send()
        .await?;

    let status = res.status();

    match status {
        reqwest::StatusCode::NO_CONTENT => Ok(None),

        s if s.is_success() => {
            let body = res.text().await?;

            info!("Poll response: {}", body);

           match serde_json::from_str::<Option<AgentTask>>(&body) {
    Ok(Some(task)) => Ok(Some(task)),
    Ok(None) => Ok(None),
    Err(e) => {
        error!("Failed to deserialize polled task: {}", e);
        error!("Raw poll response: {}", body);
        Ok(None)
    }
}
        }

        s => {
            let body = res.text().await.unwrap_or_default();
            bail!("Poll failed: HTTP {} - {}", s, body);
        }
    }
}

    pub async fn update_status(&self, id: &str, status: &str, err: Option<String>) -> Result<()> {
    let res = self
        .client
        .post(self.url(&format!("tasks/{}/status", id)))
        .json(&TaskStatusUpdate {
            status: status.to_lowercase(),
            error: err,
        })
        .send()
        .await?;

    if !res.status().is_success() {
        warn!("Status post failed: {}", res.status());
    }

    Ok(())
}

    pub async fn upload_evidence(&self, ev: &Evidence) -> Result<()> {

let raw = serde_json::to_string(&ev.raw)
    .context("Failed to serialize raw evidence")?;

let machine_id = self
    .state
    .machine_id
    .as_deref()
    .context("Backend machine_id missing; agent must register first")?
    .parse::<i64>()
    .context("Invalid backend machine_id")?;

let normalized = match &ev.normalized {
    Value::Object(map) => Some(map.clone()),
    _ => None,
};

let payload = EvidenceUpload {
    task_id: ev.task_id.clone(),
    machine_id,
    agent_id: ev.agent_id.clone(),
    collector_version: ev.collector_version.clone(),
    category: ev.category.clone(),
    raw,
    normalized,
    collected_at: ev.collected_at,
    sha256: ev.sha256.clone(),
};

    println!("UPLOAD: sending POST /evidence...");

let res = self
    .client
    .post(self.url("evidence"))
    .json(&payload)
    .send()
    .await?;

println!("UPLOAD: response received: {}", res.status());

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();

        bail!("Upload failed: HTTP {} - {}", status, body);
    }

    info!("Evidence uploaded: {}", ev.evidence_id);

    Ok(())
}

    /// Execute an AgentTask through the M4 Runtime/Dispatcher pipeline:
    /// Task -> ForensicIrDocument -> Runtime -> IrValidator -> Dispatcher -> Windows Collectors
    /// -> RawEvidence -> SHA-256 evidence packaging -> upload -> task COMPLETED
    pub async fn execute_task(&self, task: &AgentTask) -> Result<Vec<Evidence>> {
    println!("EXECUTE_TASK START: {}", task.task_id);

    info!("Executing task: {}", task.task_id);

    println!("Updating status to dispatched...");
    self.update_status(&task.task_id, "dispatched", None).await?;

    println!("Updating status to running...");
    self.update_status(&task.task_id, "running", None).await?;

    println!("Status updates done.");
    println!("Converting task to IR document...");
    
    // baaki code same

        // 1. Lower or parse task into authoritative ForensicIrDocument
        let doc = match task.to_ir_document() {
    Ok(d) => {
        println!("IR document created successfully.");
        println!("Creating runtime...");
        d
    },
            Err(e) => {
                let msg = format!("Failed to parse task IR document: {}", e);
                error!("{}", msg);
                let _ = self.update_status(&task.task_id, "FAILED", Some(msg.clone())).await;
                bail!(msg);
            }
        };

        // 2. Instantiate M4 Runtime bound to target machine identity
        let mut runtime = Runtime::with_machine_id(&self.config.machine_id);
        println!("Runtime created successfully.");

        // 3. Execute through Runtime -> IrValidator -> Dispatcher -> Windows Collectors
        println!("Starting runtime execution...");
        let raw_evidence_list = match runtime.execute(&doc) {
    Ok(ev) => {
        println!("EXECUTE_TASK: runtime returned successfully.");
        println!("Runtime execution completed. Evidence count: {}", ev.len());
        ev
    }
            Err(e) => {
                let msg = format!("Runtime execution failed: {}", e);
                error!("Task {} failed in runtime: {}", task.task_id, msg);
                let _ = self.update_status(&task.task_id, "FAILED", Some(msg.clone())).await;
                bail!(msg);
            }
        };

        // 4. Package each RawEvidence with SHA-256 and upload to /evidence
        let mut packaged_evidence = Vec::new();
        for raw in raw_evidence_list {
            println!("Packaging evidence: {}", raw.category);
            let raw_json = serde_json::to_string(&raw.payload)
    .context("Failed to serialize raw evidence")?;

let sha256 = Evidence::compute_sha256_string(&raw_json);
            let ev = Evidence {
                evidence_id: format!("ev-{}", Uuid::new_v4()),
                task_id: task.task_id.clone(),
                machine_id: self.config.machine_id.clone(),
                agent_id: self.state.agent_id.clone().unwrap_or_else(|| "unknown".into()),
                collector_version: "0.1.0".to_string(),
                category: raw.category.to_string(),
                raw: raw.payload.clone(),
                normalized: raw.payload,
                collected_at: raw.collected_at,
                sha256,
                status: EvidenceStatus::Collected,
            };

            println!("Uploading evidence: {}", ev.category);

if let Err(e) = self.upload_evidence(&ev).await {
    println!("Upload failed: {}", e);
    let msg = format!("Failed to upload evidence {}: {}", ev.evidence_id, e);
    error!("{}", msg);
    let _ = self.update_status(&task.task_id, "failed", Some(msg.clone())).await;
    bail!(msg);
}

            packaged_evidence.push(ev);
        }
        println!("All evidence uploaded. Updating task to completed...");
        // 5. Update task status to COMPLETED
        self.update_status(&task.task_id, "completed", None).await?;
        info!("Task {} completed successfully. Uploaded {} evidence items.", task.task_id, packaged_evidence.len());
        Ok(packaged_evidence)
    }
}
#[derive(Debug, Serialize)]
struct EvidenceUpload {
    task_id: String,
    machine_id: i64,
    agent_id: String,

    collector_version: String,
    category: String,

    raw: String,
    normalized: Option<serde_json::Map<String, Value>>,

    collected_at: chrono::DateTime<chrono::Utc>,
    sha256: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use compiler::ast::*;
    use compiler::ir::IrGenerator;
    use compiler::semantic::SemanticAnalyzer;
    use compiler::validator::IrValidator;
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// Spawns a lightweight local mock HTTP backend to receive agent registration, status, and evidence uploads.
    async fn spawn_mock_backend() -> (String, Arc<Mutex<usize>>, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        let upload_count = Arc::new(Mutex::new(0));
        let status_list = Arc::new(Mutex::new(Vec::new()));

        let count_clone = upload_count.clone();
        let status_clone = status_list.clone();

        let handle = tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let count_inner = count_clone.clone();
let status_inner = status_clone.clone();

tokio::spawn(async move {
    let mut buf = [0u8; 8192];
    let n = socket.read(&mut buf).await.unwrap_or(0);
    let req_str = String::from_utf8_lossy(&buf[..n]);

    if req_str.contains("POST /evidence") {
        *count_inner.lock().unwrap() += 1;
    } else if req_str.contains("/status") {
        if req_str.contains("\"running\"") || req_str.contains("\"RUNNING\"") {
            status_inner.lock().unwrap().push("RUNNING".to_string());
        } else if req_str.contains("\"completed\"") || req_str.contains("\"COMPLETED\"") {
            status_inner.lock().unwrap().push("COMPLETED".to_string());
        } else if req_str.contains("\"failed\"") || req_str.contains("\"FAILED\"") {
            status_inner.lock().unwrap().push("FAILED".to_string());
        }
    }

    let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}";
    let _ = socket.write_all(response.as_bytes()).await;
    let _ = socket.shutdown().await;
});
            }
        });

        (url, upload_count, status_list, handle)
    }

    #[tokio::test]
    async fn test_m2_m3_m4_e2e_real_windows_system_info_flow() {
        // Step 1: M2 AST definition
        let inv = Investigation {
            name: StringLiteral { value: "Real Windows System Triage".to_string(), span: Span {
    line: 1,
    column: 1,
} },
            body: vec![
                Statement::Assignment(Assignment {
                    target: Identifier { name: "sys".to_string(), span: Span {
    line: 1,
    column: 1,
} },
                    type_annotation: None,
                    value: Expr::Call(Call {
                        object: Box::new(Expr::Identifier(Identifier { name: "System".to_string(), span: Span {
    line: 1,
    column: 1,
} })),
                        function: Identifier { name: "info".to_string(), span: Span {
    line: 1,
    column: 1,
} },
                        arguments: vec![],
                        span: Span {
    line: 1,
    column: 1,
},
                    }),
                    span: Span {
    line: 1,
    column: 1,
},
                }),
            ],
            span: Span {
    line: 1,
    column: 1,
},
        };

        // Step 2: M3 Semantic Analysis
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&inv).expect("M3 Semantic analysis must succeed");

        // Step 3: M3 IR Generation
        let generator = IrGenerator::new("inv-m2-m3-m4-001").with_target_scope("WIN-E2E-NODE");
        let ir_doc = generator.generate_from_investigation(&inv).expect("M3 IR generation must succeed");

        // Step 4: M3 IR Validation
        IrValidator::validate(&ir_doc).expect("M3 IR validation must succeed");

        // Step 5: M4 Agent Task Creation
        let task = AgentTask {
            task_id: "task-real-sys-001".to_string(),
            investigation_id: ir_doc.investigation_id.clone(),
            agent_id: "agent-win-01".to_string(),
            ir: vec![],
            ir_document: Some(ir_doc),
            raw_ir: None,
            status: "PENDING".to_string(),
            created_at: Utc::now(),
        };

        // Step 6: Spawn mock HTTP backend
        let (server_url, upload_count, status_list, _server_task) = spawn_mock_backend().await;

        let config = AgentConfig {
            backend_url: server_url,
            machine_id: "WIN-E2E-NODE".to_string(),
            state_file: std::env::temp_dir().join(format!("state_{}.json", Uuid::new_v4())),
        };

        let mut agent = JockyAgent::new(config);
        agent.state.agent_id = Some("agent-win-01".to_string());

        // Step 7: M4 Agent executes task through Runtime -> Dispatcher -> Windows Collector
        let evidence_list = agent.execute_task(&task).await.expect("execute_task must succeed");

        // Step 8: Assert evidence, packaging, SHA-256, and upload
        assert_eq!(evidence_list.len(), 1, "Exactly 1 evidence item must be produced for COLLECT_SYSTEM_INFO");
        let ev = &evidence_list[0];
        assert_eq!(ev.category, "System");
        assert_eq!(ev.task_id, "task-real-sys-001");
        assert_eq!(ev.machine_id, "WIN-E2E-NODE");
        assert_eq!(ev.agent_id, "agent-win-01");

        // Verify SHA-256 hash integrity
        let expected_sha256 = Evidence::compute_sha256(&ev.raw).unwrap();
        assert_eq!(ev.sha256, expected_sha256, "Evidence sha256 must match computed sha256 over raw payload");

        // Verify real Windows OS collector fields
        assert_eq!(ev.raw["collector_name"], "WindowsSystemCollector");
        assert_eq!(ev.raw["os_version"], "Windows");
        let comp_name = ev.raw["computer_name"].as_str().unwrap();
        assert_ne!(comp_name, "UNKNOWN");
        assert!(ev.raw["total_physical_memory"].as_u64().unwrap() > 0);
        assert!(ev.raw["uptime_seconds"].as_u64().unwrap() > 0);

        // Verify HTTP uploads
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(*upload_count.lock().unwrap(), 1, "Evidence must be uploaded via HTTP POST /evidence");
        let statuses = status_list.lock().unwrap().clone();
        assert!(statuses.contains(&"RUNNING".to_string()), "Task status must transition to RUNNING");
        assert!(statuses.contains(&"COMPLETED".to_string()), "Task status must transition to COMPLETED");

        println!("=== M2+M3+M4 END-TO-END INTEGRATION TEST PASSED ===");
        println!("Computer Name : {}", comp_name);
        println!("SHA-256 Hash  : {}", ev.sha256);
        println!("Evidence ID   : {}", ev.evidence_id);
    }

    #[tokio::test]
    async fn test_execute_task_routes_through_runtime_and_rejects_invalid_ir() {
        let bad_task = AgentTask {
            task_id: "task-bad-001".to_string(),
            investigation_id: "inv-bad-001".to_string(),
            agent_id: "agent-win-01".to_string(),
            ir: vec![
                crate::evidence::ForensicIRInstruction {
                    instruction: "UNAPPROVED_ARBITRARY_OP".to_string(),
                    params: serde_json::json!({}),
                }
            ],
            ir_document: None,
            raw_ir: None,
            status: "PENDING".to_string(),
            created_at: Utc::now(),
        };

        let (server_url, _upload_count, status_list, _server_task) = spawn_mock_backend().await;

        let config = AgentConfig {
            backend_url: server_url,
            machine_id: "WIN-E2E-NODE".to_string(),
            state_file: std::env::temp_dir().join(format!("state_{}.json", Uuid::new_v4())),
        };

        let mut agent = JockyAgent::new(config);
        agent.state.agent_id = Some("agent-win-01".to_string());

        let res = agent.execute_task(&bad_task).await;
        assert!(res.is_err(), "Invalid IR must be rejected by Runtime/IrValidator");

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let statuses = status_list.lock().unwrap().clone();
        assert!(statuses.contains(&"FAILED".to_string()), "Task must be marked FAILED on validation error");
    }

    #[tokio::test]
    async fn test_execute_task_multi_instruction_pipeline_e2e() {
        let task = AgentTask {
            task_id: "task-multi-001".to_string(),
            investigation_id: "inv-multi-001".to_string(),
            agent_id: "agent-win-01".to_string(),
            ir: vec![
                crate::evidence::ForensicIRInstruction {
                    instruction: "COLLECT_PROCESSES".to_string(),
                    params: serde_json::json!({ "output_variable": "procs" }),
                },
                crate::evidence::ForensicIRInstruction {
                    instruction: "FILTER_PROCESSES".to_string(),
                    params: serde_json::json!({
                        "source_variable": "procs",
                        "output_variable": "filtered",
                        "predicate": {
                            "type": "Comparison",
                            "op": ">=",
                            "field": "pid",
                            "literal": { "type": "Integer", "value": 0 }
                        }
                    }),
                },
            ],
            ir_document: None,
            raw_ir: None,
            status: "PENDING".to_string(),
            created_at: Utc::now(),
        };

        let (server_url, upload_count, status_list, _server_task) = spawn_mock_backend().await;

        let config = AgentConfig {
            backend_url: server_url,
            machine_id: "WIN-E2E-NODE".to_string(),
            state_file: std::env::temp_dir().join(format!("state_{}.json", Uuid::new_v4())),
        };

        let mut agent = JockyAgent::new(config);
        agent.state.agent_id = Some("agent-win-01".to_string());

        let evidence_list = agent.execute_task(&task).await.expect("execute_task must succeed for multi-instruction pipeline");
        assert_eq!(evidence_list.len(), 2, "Both COLLECT_PROCESSES and FILTER_PROCESSES evidence must be returned");
        assert_eq!(evidence_list[0].category, "Process");
        assert_eq!(evidence_list[1].category, "Process");

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(*upload_count.lock().unwrap(), 2, "Both evidence items must be uploaded");
        let statuses = status_list.lock().unwrap().clone();
        assert!(statuses.contains(&"COMPLETED".to_string()));
    }
}
