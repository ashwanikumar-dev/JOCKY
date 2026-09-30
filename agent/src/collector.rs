
use crate::evidence::RawEvidence;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CollectorError {
    #[error("OS API error: {0}")]
    OsError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Unsupported operation on this OS: {0}")]
    Unsupported(String),

    #[error("Invalid collector parameters: {0}")]
    InvalidParameters(String),

    #[error("Serialization failure: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub trait Collector: Send + Sync {
    fn name(&self) -> &'static str;
    fn category(&self) -> &'static str;
    fn collect(&self, params: &Value) -> Result<RawEvidence, CollectorError>;
}