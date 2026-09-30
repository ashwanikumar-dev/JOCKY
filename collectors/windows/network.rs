use crate::collector::{Collector, CollectorError};
use crate::evidence::RawEvidence;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::borrow::Cow;
use std::net::Ipv4Addr;

#[cfg(windows)]
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCPROW_OWNER_PID, MIB_TCPTABLE_OWNER_PID, TCP_TABLE_OWNER_PID_ALL,
};
#[cfg(windows)]
use windows_sys::Win32::Networking::WinSock::AF_INET;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpEntry {
    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: String,
    pub remote_port: u16,
    pub state: String,
    pub pid: u32,
}

#[derive(Default)]
pub struct WindowsNetworkCollector {
    pub version: &'static str,
}

impl WindowsNetworkCollector {
    pub fn new() -> Self {
        Self { version: "0.1.0" }
    }

    #[cfg(windows)]
    fn collect_tcp(&self) -> Result<Vec<TcpEntry>, CollectorError> {
        let mut size = 0;
        unsafe {
            let _ = GetExtendedTcpTable(std::ptr::null_mut(), &mut size, 0, AF_INET as u32, TCP_TABLE_OWNER_PID_ALL, 0);
            if size == 0 { return Ok(vec![]); }

            let mut buf = vec![0u8; size as usize];
            if GetExtendedTcpTable(buf.as_mut_ptr() as _, &mut size, 0, AF_INET as u32, TCP_TABLE_OWNER_PID_ALL, 0) != 0 {
                return Err(CollectorError::OsError("GetExtendedTcpTable failed".into()));
            }

            let table = &*(buf.as_ptr() as *const MIB_TCPTABLE_OWNER_PID);
            let rows = table.table.as_ptr() as *const MIB_TCPROW_OWNER_PID;

            Ok((0..table.dwNumEntries as usize).map(|i| {
                let r = &*rows.add(i);
                TcpEntry {
    local_addr: Ipv4Addr::from(u32::from_be(r.dwLocalAddr)).to_string(),
    local_port: u16::from_be(r.dwLocalPort as u16),
    remote_addr: Ipv4Addr::from(u32::from_be(r.dwRemoteAddr)).to_string(),
    remote_port: u16::from_be(r.dwRemotePort as u16),
    state: match r.dwState {
        2 => "LISTEN",
        5 => "ESTABLISHED",
        11 => "TIME_WAIT",
        8 => "CLOSE_WAIT",
        _ => "OTHER",
    }
    .into(),
    pid: r.dwOwningPid,
}
            }).collect())
        }
    }

    #[cfg(not(windows))]
    fn collect_tcp(&self) -> Result<Vec<TcpEntry>, CollectorError> {
        Err(CollectorError::Unsupported("Windows only".into()))
    }
}

impl Collector for WindowsNetworkCollector {
    fn name(&self) -> &'static str { "windows_network_collector" }
    fn category(&self) -> &'static str { "Network" }

    fn collect(&self, params: &Value) -> Result<RawEvidence, CollectorError> {
        let mode = params.get("mode").and_then(|v| v.as_str()).unwrap_or("all");
        let entries = self.collect_tcp()?;

        let filtered: Vec<TcpEntry> = match mode {
            "listeners" => entries.into_iter().filter(|e| e.state == "LISTEN").collect(),
            "connections" => entries.into_iter().filter(|e| e.state != "LISTEN").collect(),
            _ => entries,
        };

        Ok(RawEvidence {
            category: Cow::Borrowed(self.category()),
            payload: json!({ "mode": mode, "count": filtered.len(), "records": filtered }),
            collected_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_network_collection_real() {
        let col = WindowsNetworkCollector::new();
        let conn = col.collect(&json!({"mode": "connections"})).expect("connections failed");
        assert_eq!(conn.category, "Network");

        let listen = col.collect(&json!({"mode": "listeners"})).expect("listeners failed");
        for r in listen.payload["records"].as_array().unwrap() {
            assert_eq!(r["state"], "LISTEN");
        }
    }
}