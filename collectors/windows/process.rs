use crate::collector::{Collector, CollectorError};
use crate::evidence::RawEvidence;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::borrow::Cow;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
#[cfg(windows)]
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
    TH32CS_SNAPPROCESS,
};
#[cfg(windows)]
use windows_sys::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessRecord {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub path: Option<String>,
    pub memory_bytes: Option<u64>,
}

pub struct WindowsProcessCollector {
    pub version: &'static str,
}

impl WindowsProcessCollector {
    pub fn new() -> Self {
        Self { version: "0.1.0" }
    }

    /// Pure in-memory filtering: Evaluates predicates without shell pipes or grep
    pub fn filter_processes(&self, list: Vec<ProcessRecord>, params: &Value) -> Vec<ProcessRecord> {
        let name_filter = params.get("name").and_then(|v| v.as_str());
        let pid_filter = params.get("pid").and_then(|v| v.as_u64()).map(|v| v as u32);
        let path_filter = params.get("path_contains").and_then(|v| v.as_str());

        list.into_iter()
            .filter(|p| {
                if let Some(target) = name_filter {
                    if !p.name.eq_ignore_ascii_case(target) { return false; }
                }
                if let Some(target_pid) = pid_filter {
                    if p.pid != target_pid { return false; }
                }
                if let Some(sub) = path_filter {
                    match &p.path {
                        Some(path) => if !path.to_ascii_lowercase().contains(&sub.to_ascii_lowercase()) { return false; },
                        None => return false,
                    }
                }
                true
            })
            .collect()
    }

    #[cfg(windows)]
    fn collect_processes_internal(&self) -> Result<Vec<ProcessRecord>, CollectorError> {
        let mut processes = Vec::with_capacity(256);

        unsafe {
            let snapshot: HANDLE = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE || snapshot == 0 {
                return Err(CollectorError::OsError(
                    "CreateToolhelp32Snapshot failed to acquire process table".into(),
                ));
            }

            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    let pid = entry.th32ProcessID;
                    let ppid = entry.th32ParentProcessID;

                    let name_len = entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len());
                    let name = String::from_utf16_lossy(&entry.szExeFile[..name_len]);

                    let (path, memory_bytes) = Self::inspect_process_details(pid);

                    processes.push(ProcessRecord { pid, ppid, name, path, memory_bytes });

                    if Process32NextW(snapshot, &mut entry) == 0 { break; }
                }
            }

            processes.shrink_to_fit();
            CloseHandle(snapshot);
        }

        Ok(processes)
    }

    #[cfg(windows)]
    fn inspect_process_details(pid: u32) -> (Option<String>, Option<u64>) {
        if pid == 0 || pid == 4 { return (None, None); }

        unsafe {
            let handle: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle == 0 || handle == INVALID_HANDLE_VALUE { return (None, None); }

            let mut path_buf = vec![0u16; 260];
            let mut size = path_buf.len() as u32;
            let path = if QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, path_buf.as_mut_ptr(), &mut size) != 0 {
                Some(String::from_utf16_lossy(&path_buf[..size as usize]))
            } else {
                None
            };

            let mut pmc: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
            let memory_bytes = if GetProcessMemoryInfo(handle, &mut pmc, std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32) != 0 {
                Some(pmc.WorkingSetSize as u64)
            } else {
                None
            };

            CloseHandle(handle);
            (path, memory_bytes)
        }
    }

    #[cfg(not(windows))]
    fn collect_processes_internal(&self) -> Result<Vec<ProcessRecord>, CollectorError> {
        Err(CollectorError::Unsupported("Windows required".into()))
    }
}

impl Default for WindowsProcessCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector for WindowsProcessCollector {
    fn name(&self) -> &'static str { "windows_process_collector" }
    fn category(&self) -> &'static str { "Process" }

    fn collect(&self, params: &Value) -> Result<RawEvidence, CollectorError> {
        let raw_processes = self.collect_processes_internal()?;
        
        let has_filter = params.get("name").is_some()
            || params.get("pid").is_some()
            || params.get("path_contains").is_some();

        let processes = if has_filter {
            self.filter_processes(raw_processes, params)
        } else {
            raw_processes
        };

        let raw_payload = json!({
            "count": processes.len(),
            "processes": processes,
            "filter_applied": has_filter,
            "collector_name": self.name(),
            "collector_version": self.version
        });

        Ok(RawEvidence {
            category: Cow::Borrowed(self.category()),
            payload: raw_payload,
            collected_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_process_collection_live() {
        let collector = WindowsProcessCollector::new();
        let result = collector.collect(&json!({})).expect("Collection failed");

        assert_eq!(result.category, "Process");
        let count = result.payload["count"].as_u64().unwrap();
        assert!(count > 0);

        let current_pid = std::process::id();
        let found_self = result.payload["processes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["pid"].as_u64().map(|v| v as u32) == Some(current_pid));

        assert!(found_self, "Must observe test runner PID: {}", current_pid);
    }

    #[test]
    fn test_process_filtering_in_memory() {
        let collector = WindowsProcessCollector::new();
        let current_pid = std::process::id();

        let result = collector.collect(&json!({ "pid": current_pid })).expect("Filter failed");
        assert_eq!(result.category, "Process");
        assert_eq!(result.payload["count"], 1);
        assert_eq!(result.payload["filter_applied"], true);
        assert_eq!(
            result.payload["processes"][0]["pid"].as_u64().map(|v| v as u32),
            Some(current_pid)
        );
    }
}