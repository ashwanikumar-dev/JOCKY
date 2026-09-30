use std::borrow::Cow;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use windows_sys::Win32::System::SystemInformation::{
    GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX,
};
use windows_sys::Win32::System::WindowsProgramming::GetComputerNameW;

// Import trait from collector, and RawEvidence directly from evidence
use crate::collector::{Collector, CollectorError};
use crate::evidence::RawEvidence;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTelemetry {
    pub os_version: String,
    pub os_release: String,
    pub uptime_seconds: u64,
    pub computer_name: String,
    pub architecture: String,
    pub total_physical_memory: u64,
}

pub struct WindowsSystemCollector {
    pub version: &'static str,
}

impl WindowsSystemCollector {
    pub fn new() -> Self {
        Self {
            version: "0.1.0",
        }
    }

    pub fn get_uptime_seconds(&self) -> u64 {
        unsafe { GetTickCount64() / 1000 }
    }

    pub fn get_os_release(&self) -> String {
        format!("Windows NT {}", std::env::consts::OS)
    }
}

impl Collector for WindowsSystemCollector {
    fn name(&self) -> &'static str {
        "WindowsSystemCollector"
    }

    fn category(&self) -> &'static str {
        "System"
    }

    fn collect(&self, _params: &Value) -> Result<RawEvidence, CollectorError> {
        let mut computer_name = String::from("UNKNOWN");
        let mut buf = [0u16; 256];
        let mut size = buf.len() as u32;

        unsafe {
            if GetComputerNameW(buf.as_mut_ptr(), &mut size) != 0 {
                computer_name = OsString::from_wide(&buf[..size as usize])
                    .to_string_lossy()
                    .into_owned();
            }
        }

        let mut mem_status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
        mem_status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        let mut total_mem = 0u64;

        unsafe {
            if GlobalMemoryStatusEx(&mut mem_status) != 0 {
                total_mem = mem_status.ullTotalPhys;
            }
        }

        let uptime_seconds = self.get_uptime_seconds();
        let os_release = self.get_os_release();

        Ok(RawEvidence {
            category: Cow::Borrowed(self.category()),
            payload: json!({
                "collector_name": self.name(),
                "collector_version": self.version,
                "os_version": "Windows",
                "os_release": os_release,
                "uptime_seconds": uptime_seconds,
                "computer_name": computer_name,
                "architecture": std::env::consts::ARCH,
                "total_physical_memory": total_mem,
            }),
            collected_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_system_collection_real() {
        let col = WindowsSystemCollector::new();
        let evidence = col.collect(&json!({})).expect("failed system collection");
        assert_ne!(evidence.payload["computer_name"], "UNKNOWN");
        assert!(evidence.payload["total_physical_memory"].as_u64().unwrap() > 0);
    }

    #[test]
    fn test_windows_system_consecutive_10_runs() {
        let col = WindowsSystemCollector::new();
        for i in 1..=10 {
            let res = col.collect(&json!({})).unwrap_or_else(|e| panic!("Run {} failed: {:?}", i, e));
            assert_ne!(res.payload["computer_name"], "UNKNOWN", "Run {} invalid computer_name", i);
            assert!(res.payload["total_physical_memory"].as_u64().unwrap() > 0, "Run {} zero memory", i);
            assert!(res.payload["uptime_seconds"].as_u64().is_some(), "Run {} missing uptime", i);
        }
    }
}