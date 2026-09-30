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

pub struct Dispatcher {
    pub machine_id: String,
    pub system: WindowsSystemCollector,
    pub process: WindowsProcessCollector,
    pub network: WindowsNetworkCollector,
    pub runtime_memory: HashMap<String, Value>,
}

impl Dispatcher {
    pub fn new() -> Self {
        let default_id = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "WIN-ENDPOINT".to_string());

        Self::with_machine_id(default_id)
    }

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

    pub fn dispatch_instruction(
        &mut self,
        inst: &IrInstruction,
    ) -> Result<Option<RawEvidence>, DispatcherError> {
        match inst.op.as_str() {
            // ---------------------------------------------------------
            // Investigation boundaries
            // ---------------------------------------------------------
            "INVESTIGATION_BEGIN" => {
                self.runtime_memory.clear();
                Ok(None)
            }

            "INVESTIGATION_END" => Ok(None),

            // ---------------------------------------------------------
            // System collection
            // ---------------------------------------------------------
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

            // ---------------------------------------------------------
            // Process collection
            // ---------------------------------------------------------
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

            // ---------------------------------------------------------
            // Process filtering
            // ---------------------------------------------------------
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
                        self.process
                            .filter_processes(processes, predicate_val)
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

            // ---------------------------------------------------------
            // Network connections
            // ---------------------------------------------------------
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

            // ---------------------------------------------------------
            // Network listeners
            // ---------------------------------------------------------
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

            // ---------------------------------------------------------
            // Evidence preservation
            // ---------------------------------------------------------
            "PRESERVE_EVIDENCE" => {
                let evidence_tag = self.get_param_str(inst, "evidence_tag")?;
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

                self.runtime_memory.insert(evidence_tag, payload);

                Ok(Some(evidence))
            }

            // ---------------------------------------------------------
            // Evidence verification
            // ---------------------------------------------------------
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

                // Cryptographic verification happens in the evidence
                // packaging/upload layer. At dispatcher level we verify
                // that the referenced evidence exists in runtime memory.
                Ok(None)
            }

            // ---------------------------------------------------------
            // Unknown operation
            // ---------------------------------------------------------
            unsupported => {
                Err(DispatcherError::UnsupportedOp(
                    unsupported.to_string(),
                ))
            }
        }
    }

    pub fn dispatch_ir(
        &mut self,
        doc: &ForensicIrDocument,
    ) -> Result<Vec<RawEvidence>, DispatcherError> {
        self.runtime_memory.clear();

        let mut collected = Vec::new();

        for inst in &doc.instructions {
            if let Some(evidence) = self.dispatch_instruction(inst)? {
                collected.push(evidence);
            }
        }

        Ok(collected)
    }

    pub fn dispatch_ir_json(
        &mut self,
        ir_json: &str,
    ) -> Result<Vec<RawEvidence>, DispatcherError> {
        let doc: ForensicIrDocument = serde_json::from_str(ir_json)?;

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

// =====================================================================
// Predicate evaluation
// =====================================================================

// =====================================================================
// Predicate evaluation
// =====================================================================

fn evaluate_predicate(
    predicate: &Predicate,
    process: &ProcessRecord,
) -> bool {
    match predicate {
        Predicate::LogicalAnd { left, right } => {
            evaluate_predicate(left, process)
                && evaluate_predicate(right, process)
        }

        Predicate::LogicalOr { left, right } => {
            evaluate_predicate(left, process)
                || evaluate_predicate(right, process)
        }

        Predicate::LogicalNot { operand } => {
            !evaluate_predicate(operand, process)
        }

        Predicate::Comparison {
            op,
            field,
            literal,
        } => {
            let actual = process_field_value(process, field);
            compare_values(&actual, op, literal)
        }
    }
}

fn process_field_value(
    process: &ProcessRecord,
    field: &str,
) -> Value {
    match field {
        "pid" => json!(process.pid),
        "ppid" => json!(process.ppid),
        "name" => json!(process.name),
        "path" => json!(process.path),
        "memory_bytes" => json!(process.memory_bytes),
        _ => Value::Null,
    }
}

fn compare_values(
    actual: &Value,
    operator: &str,
    expected: &PredicateLiteral,
) -> bool {
    match expected {
        PredicateLiteral::String { value } => {
            let Some(actual) = actual.as_str() else {
                return false;
            };

            compare_ord(operator, actual, value)
        }

        PredicateLiteral::Integer { value } => {
            let Some(actual) = actual.as_i64() else {
                return false;
            };

            compare_ord(operator, actual, *value)
        }

        PredicateLiteral::Float { value } => {
            let Some(actual) = actual.as_f64() else {
                return false;
            };

            compare_partial(operator, actual, *value)
        }

        PredicateLiteral::Boolean { value } => {
            let Some(actual) = actual.as_bool() else {
                return false;
            };

            match operator {
                "==" => actual == *value,
                "!=" => actual != *value,
                _ => false,
            }
        }

        PredicateLiteral::ByteSize { value_bytes, .. } => {
            let Some(actual) = actual.as_u64() else {
                return false;
            };

            compare_ord(operator, actual, *value_bytes)
        }
    }
}

fn compare_ord<T: PartialOrd + PartialEq>(
    operator: &str,
    left: T,
    right: T,
) -> bool {
    match operator {
        "==" => left == right,
        "!=" => left != right,
        ">" => left > right,
        "<" => left < right,
        ">=" => left >= right,
        "<=" => left <= right,
        _ => false,
    }
}

fn compare_partial(
    operator: &str,
    left: f64,
    right: f64,
) -> bool {
    match operator {
        "==" => left == right,
        "!=" => left != right,
        ">" => left > right,
        "<" => left < right,
        ">=" => left >= right,
        "<=" => left <= right,
        _ => false,
    }
}