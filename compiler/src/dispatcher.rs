use std::borrow::Cow;
use std::collections::HashMap;

use chrono::Utc;
use serde_json::{json, Value};
use thiserror::Error;

use crate::collector::Collector;
use crate::evidence::RawEvidence;
use crate::windows_collectors::{
    network::WindowsNetworkCollector,
    process::{ProcessRecord, WindowsProcessCollector},
    system::WindowsSystemCollector,
};

use compiler::ir::{ForensicIrDocument, IrInstruction, Predicate, PredicateLiteral};

#[derive(Debug, Error)]
pub enum DispatcherError {
    #[error("IR parsing error: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Unsupported or unknown IR opcode: '{0}'")]
    UnsupportedOp(String),

    #[error("Missing required parameter '{param}' in instruction '{op}'")]
    MissingParameter { op: String, param: String },

    #[error("Source variable '{0}' not found in runtime memory")]
    MissingSourceVariable(String),

    #[error("Execution halted: {0}")]
    ExecutionHalted(String),

    #[error("Collector error: {0}")]
    CollectorError(String),
}

/// Runtime Dispatcher that processes Forensic IR instructions by routing
/// them directly to endpoint collectors (System, Process, Network).
pub struct Dispatcher {
    pub machine_id: String,
    pub system: WindowsSystemCollector,
    pub process: WindowsProcessCollector,
    pub network: WindowsNetworkCollector,
    pub runtime_memory: HashMap<String, Value>,
}

impl Dispatcher {
    /// Create a new Dispatcher with default local Windows collectors.
    pub fn new() -> Self {
        let default_id = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "WIN-ENDPOINT".to_string());

        Self::with_machine_id(default_id)
    }

    /// Create a Dispatcher with an explicit machine identity.
    pub fn with_machine_id(machine_id: impl Into<String>) -> Self {
        Self {
            machine_id: machine_id.into(),
            system: WindowsSystemCollector::new(),
            process: WindowsProcessCollector::new(),
            network: WindowsNetworkCollector::new(),
            runtime_memory: HashMap::new(),
        }
    }

    fn tag_machine_id(&self, evidence: &mut RawEvidence) {
        if let Value::Object(ref mut map) = evidence.payload {
            if !map.contains_key("machine_id") {
                map.insert(
                    "machine_id".to_string(),
                    Value::String(self.machine_id.clone()),
                );
            }
        }
    }

    /// Dispatch a single IR instruction through the collector boundary.
    pub fn dispatch_instruction(
        &mut self,
        inst: &IrInstruction,
    ) -> Result<Option<RawEvidence>, DispatcherError> {
        match inst.op.as_str() {
            "INVESTIGATION_BEGIN" => {
                // Initialize clean, isolated execution state for this investigation.
                self.runtime_memory.clear();
                Ok(None)
            }

            "COLLECT_SYSTEM_INFO" => {
                let out_var = self.get_param_str(inst, "output_variable")?;
                let params = Value::Object(inst.params.clone());

                let mut evidence = self
                    .system
                    .collect(&params)
                    .map_err(|e| DispatcherError::CollectorError(e.to_string()))?;

                self.tag_machine_id(&mut evidence);

                self.runtime_memory
                    .insert(out_var, evidence.payload.clone());

                Ok(Some(evidence))
            }

            "COLLECT_PROCESSES" => {
                let out_var = self.get_param_str(inst, "output_variable")?;
                let params = Value::Object(inst.params.clone());

                let mut evidence = self
                    .process
                    .collect(&params)
                    .map_err(|e| DispatcherError::CollectorError(e.to_string()))?;

                self.tag_machine_id(&mut evidence);

                self.runtime_memory
                    .insert(out_var, evidence.payload.clone());

                Ok(Some(evidence))
            }

            "FILTER_PROCESSES" => {
                let source_var = self.get_param_str(inst, "source_variable")?;
                let out_var = self.get_param_str(inst, "output_variable")?;

                let source_payload = self
                    .runtime_memory
                    .get(&source_var)
                    .ok_or_else(|| {
                        DispatcherError::MissingSourceVariable(source_var.clone())
                    })?;

                let processes_json = source_payload
                    .get("processes")
                    .ok_or_else(|| {
                        DispatcherError::ExecutionHalted(format!(
                            "Variable '{}' contains no processes array",
                            source_var
                        ))
                    })?;

                let processes: Vec<ProcessRecord> =
                    serde_json::from_value(processes_json.clone())?;

                let predicate_val = inst.params.get("predicate").ok_or_else(|| {
                    DispatcherError::MissingParameter {
                        op: inst.op.clone(),
                        param: "predicate".to_string(),
                    }
                })?;

                let filtered: Vec<ProcessRecord> =
                    if let Ok(predicate) =
                        serde_json::from_value::<Predicate>(predicate_val.clone())
                    {
                        processes
                            .into_iter()
                            .filter(|p| evaluate_predicate(&predicate, p))
                            .collect()
                    } else {
                        self.process.filter_processes(
                            processes,
                            predicate_val,
                        )
                    };

                let raw_payload = json!({
                    "count": filtered.len(),
                    "processes": filtered,
                    "filter_applied": true,
                    "collector_name": self.process.name(),
                    "collector_version": self.process.version,
                    "machine_id": self.machine_id,
                });

                let mut evidence = RawEvidence {
                    category: Cow::Borrowed(self.process.category()),
                    payload: raw_payload,
                    collected_at: Utc::now(),
                };

                self.tag_machine_id(&mut evidence);

                self.runtime_memory
                    .insert(out_var, evidence.payload.clone());

                Ok(Some(evidence))
            }

            "COLLECT_NETWORK_CONNECTIONS" => {
                let out_var = self.get_param_str(inst, "output_variable")?;

                let params = json!({
                    "mode": "connections"
                });

                let mut evidence = self
                    .network
                    .collect(&params)
                    .map_err(|e| DispatcherError::CollectorError(e.to_string()))?;

                self.tag_machine_id(&mut evidence);

                self.runtime_memory
                    .insert(out_var, evidence.payload.clone());

                Ok(Some(evidence))
            }

            "COLLECT_NETWORK_LISTENERS" => {
                let out_var = self.get_param_str(inst, "output_variable")?;

                let params = json!({
                    "mode": "listeners"
                });

                let mut evidence = self
                    .network
                    .collect(&params)
                    .map_err(|e| DispatcherError::CollectorError(e.to_string()))?;

                self.tag_machine_id(&mut evidence);

                self.runtime_memory
                    .insert(out_var, evidence.payload.clone());

                Ok(Some(evidence))
            }

            "PRESERVE_EVIDENCE" => {
                let evidence_tag =
                    self.get_param_str(inst, "evidence_tag")?;

                let source_variable =
                    self.get_param_str(inst, "source_variable")?;

                let payload = self
                    .runtime_memory
                    .get(&source_variable)
                    .cloned()
                    .ok_or_else(|| {
                        DispatcherError::MissingSourceVariable(
                            source_variable.clone(),
                        )
                    })?;

                let evidence = RawEvidence {
                    category: Cow::Owned(evidence_tag.clone()),
                    payload: payload.clone(),
                    collected_at: Utc::now(),
                };

                self.runtime_memory
                    .insert(evidence_tag, payload);

                Ok(Some(evidence))
            }

            "VERIFY_EVIDENCE" => {
                let evidence_variable =
                    self.get_param_str(inst, "evidence_variable")?;

                if !self
                    .runtime_memory
                    .contains_key(&evidence_variable)
                {
                    return Err(
                        DispatcherError::MissingSourceVariable(
                            evidence_variable,
                        ),
                    );
                }

                // Verification is currently a runtime existence/integrity
                // boundary. Cryptographic verification is handled by the
                // evidence layer after packaging/upload.
                Ok(None)
            }

            "INVESTIGATION_END" => Ok(None),

            unsupported => {
                Err(DispatcherError::UnsupportedOp(
                    unsupported.to_string(),
                ))
            }
        }
    }

    /// Dispatch all instructions in an IR document sequentially.
    pub fn dispatch_ir(
        &mut self,
        doc: &ForensicIrDocument,
    ) -> Result<Vec<RawEvidence>, DispatcherError> {
        let mut collected = Vec::new();

        for inst in &doc.instructions {
            if let Some(evidence) =
                self.dispatch_instruction(inst)?
            {
                collected.push(evidence);
            }
        }

        Ok(collected)
    }

    /// Parse JSON IR and dispatch.
    pub fn dispatch_ir_json(
        &mut self,
        ir_json: &str,
    ) -> Result<Vec<RawEvidence>, DispatcherError> {
        let doc: ForensicIrDocument =
            serde_json::from_str(ir_json)?;

        self.dispatch_ir(&doc)
    }

    fn get_param_str(
        &self,
        inst: &IrInstruction,
        key: &str,
    ) -> Result<String, DispatcherError> {
        inst.params
            .get(key)
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .ok_or_else(|| DispatcherError::MissingParameter {
                op: inst.op.clone(),
                param: key.to_string(),
            })
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluates a Forensic IR filter predicate against a ProcessRecord in memory.
fn evaluate_predicate(
    pred: &Predicate,
    proc: &ProcessRecord,
) -> bool {
    match pred {
        Predicate::LogicalAnd { left, right } => {
            evaluate_predicate(left, proc)
                && evaluate_predicate(right, proc)
        }

        Predicate::LogicalOr { left, right } => {
            evaluate_predicate(left, proc)
                || evaluate_predicate(right, proc)
        }

        Predicate::LogicalNot { operand } => {
            !evaluate_predicate(operand, proc)
        }

        Predicate::Comparison {
            op,
            field,
            literal,
        } => evaluate_comparison(op, field, literal, proc),
    }
}

fn evaluate_comparison(
    op: &str,
    field: &str,
    literal: &PredicateLiteral,
    proc: &ProcessRecord,
) -> bool {
    match field {
        "name" => {
            if let PredicateLiteral::String { value } = literal {
                match op {
                    "==" => proc.name.eq_ignore_ascii_case(value),
                    "!=" => !proc.name.eq_ignore_ascii_case(value),
                    _ => false,
                }
            } else {
                false
            }
        }

        "pid" => {
            if let PredicateLiteral::Integer { value } = literal {
                compare_ord(op, proc.pid as i64, *value)
            } else {
                false
            }
        }

        "ppid" => {
            if let PredicateLiteral::Integer { value } = literal {
                compare_ord(op, proc.ppid as i64, *value)
            } else {
                false
            }
        }

        "memory" => {
            let proc_mem = proc.memory_bytes.unwrap_or(0);

            let target_bytes = match literal {
                PredicateLiteral::ByteSize {
                    value_bytes, ..
                } => *value_bytes,

                PredicateLiteral::Integer { value } => {
                    if *value < 0 {
                        return false;
                    }

                    *value as u64
                }

                _ => return false,
            };

            compare_ord(op, proc_mem, target_bytes)
        }

        "path" => {
            if let PredicateLiteral::String { value } = literal {
                let proc_path =
                    proc.path.as_deref().unwrap_or("");

                match op {
                    "==" => proc_path.eq_ignore_ascii_case(value),
                    "!=" => !proc_path.eq_ignore_ascii_case(value),
                    _ => false,
                }
            } else {
                false
            }
        }

        _ => false,
    }
}

fn compare_ord<T: Ord>(
    op: &str,
    left: T,
    right: T,
) -> bool {
    match op {
        "==" => left == right,
        "!=" => left != right,
        ">" => left > right,
        "<" => left < right,
        ">=" => left >= right,
        "<=" => left <= right,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOCK_SYS_IR_JSON: &str = r#"{
      "ir_version": "v0.1.0",
      "investigation_id": "inv-2026-sih-001",
      "investigation_name": "Endpoint Triage",
      "target_scope": "ALL",
      "instructions": [
        {
          "sequence": 1,
          "op": "INVESTIGATION_BEGIN",
          "params": {
            "investigation_id": "inv-2026-sih-001",
            "investigation_name": "Endpoint Triage",
            "timestamp_utc": "2026-09-24T13:46:00Z"
          }
        },
        {
          "sequence": 2,
          "op": "COLLECT_SYSTEM_INFO",
          "params": {
            "output_variable": "sys"
          }
        },
        {
          "sequence": 3,
          "op": "INVESTIGATION_END",
          "params": {
            "investigation_id": "inv-2026-sih-001",
            "status": "SUCCESS"
          }
        }
      ]
    }"#;

    #[test]
    fn test_dispatcher_with_real_collector() {
        let mut dispatcher = Dispatcher::new();

        let results = dispatcher
            .dispatch_ir_json(MOCK_SYS_IR_JSON)
            .expect("Dispatch should succeed");

        assert_eq!(results.len(), 1);

        let ev = &results[0];

        assert_eq!(ev.category, "System");

        let payload = &ev.payload;

        assert_eq!(
            payload["collector_name"],
            "WindowsSystemCollector"
        );

        assert_eq!(
            payload["os_version"],
            "Windows"
        );

        assert_ne!(
            payload["computer_name"],
            "UNKNOWN"
        );

        assert!(
            payload["total_physical_memory"]
                .as_u64()
                .unwrap()
                > 0
        );

        assert!(
            payload["uptime_seconds"]
                .as_u64()
                .is_some()
        );
    }

    #[test]
    fn test_dispatcher_collect_processes() {
        let mut dispatcher = Dispatcher::new();

        let doc = ForensicIrDocument {
            ir_version: "v0.1.0".to_string(),
            investigation_id: "inv-proc-01".to_string(),
            investigation_name: "Process Triage".to_string(),
            target_scope: "WIN-01".to_string(),
            instructions: vec![
                IrInstruction {
                    sequence: 1,
                    op: "INVESTIGATION_BEGIN".to_string(),
                    params: [
                        (
                            "investigation_id".to_string(),
                            json!("inv-proc-01"),
                        ),
                        (
                            "investigation_name".to_string(),
                            json!("Process Triage"),
                        ),
                        (
                            "timestamp_utc".to_string(),
                            json!("2026-09-24T13:46:00Z"),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                },
                IrInstruction {
                    sequence: 2,
                    op: "COLLECT_PROCESSES".to_string(),
                    params: [
                        (
                            "output_variable".to_string(),
                            json!("procs"),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                },
                IrInstruction {
                    sequence: 3,
                    op: "INVESTIGATION_END".to_string(),
                    params: [
                        (
                            "investigation_id".to_string(),
                            json!("inv-proc-01"),
                        ),
                        (
                            "status".to_string(),
                            json!("SUCCESS"),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                },
            ],
        };

        let results = dispatcher
            .dispatch_ir(&doc)
            .expect("COLLECT_PROCESSES dispatch failed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].category, "Process");

        let count = results[0]["count"]
            .as_u64()
            .unwrap();

        assert!(count > 0);
    }

    #[test]
    fn test_dispatcher_filter_processes() {
        let mut dispatcher = Dispatcher::new();

        dispatcher.runtime_memory.insert(
            "procs".to_string(),
            json!({
                "count": 3,
                "processes": [
                    {
                        "pid": 100,
                        "ppid": 4,
                        "name": "powershell.exe",
                        "path": "C:\\Windows\\System32\\powershell.exe",
                        "memory_bytes": 600000000u64
                    },
                    {
                        "pid": 200,
                        "ppid": 4,
                        "name": "cmd.exe",
                        "path": "C:\\Windows\\System32\\cmd.exe",
                        "memory_bytes": 50000000u64
                    },
                    {
                        "pid": 300,
                        "ppid": 4,
                        "name": "notepad.exe",
                        "path": "C:\\Windows\\System32\\notepad.exe",
                        "memory_bytes": 30000000u64
                    }
                ]
            }),
        );

        let filter_inst = IrInstruction {
            sequence: 1,
            op: "FILTER_PROCESSES".to_string(),
            params: [
                (
                    "source_variable".to_string(),
                    json!("procs"),
                ),
                (
                    "output_variable".to_string(),
                    json!("flagged"),
                ),
                (
                    "predicate".to_string(),
                    json!({
                        "type": "Comparison",
                        "op": "==",
                        "field": "name",
                        "literal": {
                            "type": "String",
                            "value": "powershell.exe"
                        }
                    }),
                ),
            ]
            .into_iter()
            .collect(),
        };

        let res = dispatcher
            .dispatch_instruction(&filter_inst)
            .expect("FILTER_PROCESSES failed");

        assert!(res.is_some());

        let ev = res.unwrap();

        assert_eq!(ev.category, "Process");
        assert_eq!(ev.payload["count"], 1);
        assert_eq!(
            ev.payload["processes"][0]["name"],
            "powershell.exe"
        );
    }

    #[test]
    fn test_dispatcher_collect_network_connections_and_listeners() {
        let mut dispatcher = Dispatcher::new();

        let inst_conn = IrInstruction {
            sequence: 1,
            op: "COLLECT_NETWORK_CONNECTIONS".to_string(),
            params: [
                (
                    "output_variable".to_string(),
                    json!("conns"),
                ),
            ]
            .into_iter()
            .collect(),
        };

        let res_conn = dispatcher
            .dispatch_instruction(&inst_conn)
            .expect("COLLECT_NETWORK_CONNECTIONS failed");

        assert!(res_conn.is_some());

        let ev_conn = res_conn.unwrap();

        assert_eq!(ev_conn.category, "Network");
        assert_eq!(ev_conn.payload["mode"], "connections");

        let inst_listen = IrInstruction {
            sequence: 2,
            op: "COLLECT_NETWORK_LISTENERS".to_string(),
            params: [
                (
                    "output_variable".to_string(),
                    json!("listeners"),
                ),
            ]
            .into_iter()
            .collect(),
        };

        let res_listen = dispatcher
            .dispatch_instruction(&inst_listen)
            .expect("COLLECT_NETWORK_LISTENERS failed");

        assert!(res_listen.is_some());

        let ev_listen = res_listen.unwrap();

        assert_eq!(ev_listen.category, "Network");
        assert_eq!(ev_listen.payload["mode"], "listeners");
    }

    #[test]
    fn test_dispatcher_rejects_unsupported_op() {
        let bad_ir = r#"{
          "ir_version": "v0.1.0",
          "investigation_id": "bad-01",
          "investigation_name": "Malicious Instruction",
          "target_scope": "ALL",
          "instructions": [
            {
              "sequence": 1,
              "op": "DROP_DATABASE",
              "params": {}
            }
          ]
        }"#;

        let mut dispatcher = Dispatcher::new();

        let result = dispatcher.dispatch_ir_json(bad_ir);

        assert!(result.is_err());

        match result.unwrap_err() {
            DispatcherError::UnsupportedOp(op) => {
                assert_eq!(op, "DROP_DATABASE")
            }

            other => panic!(
                "Expected UnsupportedOp error, got: {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_dispatcher_filter_processes_by_ppid() {
        let mut dispatcher = Dispatcher::new();

        dispatcher.runtime_memory.insert(
            "procs".to_string(),
            json!({
                "count": 3,
                "processes": [
                    {
                        "pid": 100,
                        "ppid": 4,
                        "name": "powershell.exe",
                        "path": "C:\\Windows\\System32\\powershell.exe",
                        "memory_bytes": 600000000u64
                    },
                    {
                        "pid": 200,
                        "ppid": 4,
                        "name": "cmd.exe",
                        "path": "C:\\Windows\\System32\\cmd.exe",
                        "memory_bytes": 50000000u64
                    },
                    {
                        "pid": 300,
                        "ppid": 1,
                        "name": "notepad.exe",
                        "path": "C:\\Windows\\System32\\notepad.exe",
                        "memory_bytes": 30000000u64
                    }
                ]
            }),
        );

        let filter_inst = IrInstruction {
            sequence: 1,
            op: "FILTER_PROCESSES".to_string(),
            params: [
                (
                    "source_variable".to_string(),
                    json!("procs"),
                ),
                (
                    "output_variable".to_string(),
                    json!("flagged"),
                ),
                (
                    "predicate".to_string(),
                    json!({
                        "type": "Comparison",
                        "op": "==",
                        "field": "ppid",
                        "literal": {
                            "type": "Integer",
                            "value": 4
                        }
                    }),
                ),
            ]
            .into_iter()
            .collect(),
        };

        let res = dispatcher
            .dispatch_instruction(&filter_inst)
            .expect("FILTER_PROCESSES failed");

        assert!(res.is_some());

        let ev = res.unwrap();

        assert_eq!(ev.payload["count"], 2);
    }

    #[test]
    fn test_dispatcher_rejects_empty_output_variable() {
        let mut dispatcher = Dispatcher::new();

        let inst = IrInstruction {
            sequence: 1,
            op: "COLLECT_SYSTEM_INFO".to_string(),
            params: [
                (
                    "output_variable".to_string(),
                    json!("   "),
                ),
            ]
            .into_iter()
            .collect(),
        };

        let res = dispatcher.dispatch_instruction(&inst);

        assert!(
            res.is_err(),
            "Empty or whitespace output_variable must be rejected"
        );

        match res.unwrap_err() {
            DispatcherError::MissingParameter { op, param } => {
                assert_eq!(op, "COLLECT_SYSTEM_INFO");
                assert_eq!(param, "output_variable");
            }

            other => panic!(
                "Expected MissingParameter, got: {:?}",
                other
            ),
        }
    }
}