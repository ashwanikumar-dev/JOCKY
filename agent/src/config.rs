use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub backend_url: String,
    pub machine_id: String,
    pub state_file: PathBuf,
}

impl Default for AgentConfig {
    fn default() -> Self {
        let hostname = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "WIN-ENDPOINT".to_string());

        Self {
            backend_url: std::env::var("JOCKY_BACKEND_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string()),
            machine_id: hostname,
            state_file: PathBuf::from("agent_state.json"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentState {
    pub agent_id: Option<String>,
    pub machine_id: Option<String>,
}

impl AgentState {
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let serialized = serde_json::to_string_pretty(self)?;
        fs::write(path, serialized)
    }
}