use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::borrow::Cow;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    Collected,
    Uploaded,
    Verified,
    IntegrityFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEvidence {
    pub category: Cow<'static, str>,
    pub payload: Value,
    pub collected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_id: String,
    pub task_id: String,
    pub machine_id: String,
    pub agent_id: String,
    pub collector_version: String,
    pub category: String,
    pub raw: Value,
    pub normalized: Value,
    pub collected_at: DateTime<Utc>,
    pub sha256: String,
    pub status: EvidenceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicIRInstruction {
    pub instruction: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub task_id: String,
    pub investigation_id: String,
    pub agent_id: String,
    #[serde(default)]
    pub ir: Vec<ForensicIRInstruction>,
    #[serde(default)]
    pub ir_document: Option<compiler::ir::ForensicIrDocument>,
    #[serde(default)]
    pub raw_ir: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl AgentTask {
    /// Convert task IR into an authoritative compiler ForensicIrDocument.
    pub fn to_ir_document(&self) -> Result<compiler::ir::ForensicIrDocument, anyhow::Error> {
        if let Some(ref doc) = self.ir_document {
            return Ok(doc.clone());
        }

        if let Some(ref raw_json) = self.raw_ir {
            let doc: compiler::ir::ForensicIrDocument = serde_json::from_str(raw_json)
                .map_err(|e| anyhow::anyhow!("Failed to deserialize task raw_ir JSON into ForensicIrDocument: {}", e))?;
            return Ok(doc);
        }

        // Lower from self.ir (Vec<ForensicIRInstruction>)
        let mut instructions = Vec::new();
        let has_begin = self.ir.first().map_or(false, |i| i.instruction == "INVESTIGATION_BEGIN");
        let has_end = self.ir.last().map_or(false, |i| i.instruction == "INVESTIGATION_END");

        let mut seq = 1;
        if !has_begin {
            let mut begin_params = serde_json::Map::new();
            begin_params.insert("investigation_id".to_string(), serde_json::json!(self.investigation_id));
            begin_params.insert("investigation_name".to_string(), serde_json::json!(format!("Task {}", self.task_id)));
            begin_params.insert("timestamp_utc".to_string(), serde_json::json!(self.created_at.to_rfc3339()));
            instructions.push(compiler::ir::IrInstruction {
                sequence: seq,
                op: "INVESTIGATION_BEGIN".to_string(),
                params: begin_params,
            });
            seq += 1;
        }

        for item in &self.ir {
            let params = match &item.params {
                Value::Object(map) => map.clone(),
                _ => serde_json::Map::new(),
            };
            instructions.push(compiler::ir::IrInstruction {
                sequence: seq,
                op: item.instruction.clone(),
                params,
            });
            seq += 1;
        }

        if !has_end {
            let mut end_params = serde_json::Map::new();
            end_params.insert("investigation_id".to_string(), serde_json::json!(self.investigation_id));
            end_params.insert("status".to_string(), serde_json::json!("SUCCESS"));
            instructions.push(compiler::ir::IrInstruction {
                sequence: seq,
                op: "INVESTIGATION_END".to_string(),
                params: end_params,
            });
        }

        Ok(compiler::ir::ForensicIrDocument {
            ir_version: "v0.1.0".to_string(),
            investigation_id: self.investigation_id.clone(),
            investigation_name: format!("Task {}", self.task_id),
            target_scope: self.agent_id.clone(),
            instructions,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatusUpdate {
    pub status: String,
    #[serde(default)]
    pub error: Option<String>,
}

impl Evidence {
    /// Computes the SHA-256 hash over raw payload bytes for tamper detection.
    pub fn compute_sha256(raw_payload: &Value) -> Result<String, serde_json::Error> {
        let serialized = serde_json::to_vec(raw_payload)?;
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        Ok(format!("{:x}", hasher.finalize()))
    }
    pub fn compute_sha256_string(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())
}
}