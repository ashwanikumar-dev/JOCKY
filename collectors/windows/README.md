# Windows Native Telemetry Collectors

## Overview
The JOCKY Windows agent implements direct Win32 API telemetry collection through `windows-sys`. All collectors strictly follow the frozen monorepo contracts, packaging collected data into `RawEvidence` structs with UTC ISO 8601 timestamps and SHA-256 integrity digests.

---

## 1. System Collector (`WindowsSystemCollector`)
* **Category**: `"System"`
* **Collector Version**: `0.1.0`
* **Win32 APIs**:
  * `GetComputerNameW`: NetBIOS and machine identifier resolution.
  * `GlobalMemoryStatusEx`: Physical memory capacity inspection via `ullTotalPhys`.
  * `GetTickCount64`: Monotonic boot uptime tracking.
* **Payload Schema**:
```json
{
  "collector_name": "WindowsSystemCollector",
  "collector_version": "0.1.0",
  "os_version": "Windows",
  "os_release": "Windows NT windows",
  "uptime_seconds": 3600,
  "computer_name": "DESKTOP-WIN",
  "architecture": "x86_64",
  "total_physical_memory": 17179869184
}
```

---

## 2. Process Collector (`WindowsProcessCollector`)
* **Category**: `"Process"`
* **Collector Version**: `0.1.0`
* **Win32 APIs**:
  * `CreateToolhelp32Snapshot` (`TH32CS_SNAPPROCESS`): Snapshot of current process table.
  * `Process32FirstW` / `Process32NextW`: Traversal of active process entries.
  * `OpenProcess` (`PROCESS_QUERY_LIMITED_INFORMATION`): Acquisition of read-only process handles.
  * `QueryFullProcessImageNameW`: Normalization of executable image disk paths.
* **Resilience & Timing Handling**:
  * **Transient Snapshot Errors**: Retries snapshot creation up to 3 times with a 10ms backoff interval.
  * **Permission Degradation (`ERROR_ACCESS_DENIED`)**: Gracefully degrades for protected processes (PID 0, PID 4, and protected services) by preserving executable names and assigning `path: "<access_denied>"` instead of halting execution.
* **Supported IR Query Parameters**:
  * `name`: Substring match on executable name (case-insensitive).
  * `pid`: Exact PID integer match.
  * `path_contains`: Substring match on the absolute executable path.
* **Payload Schema**:
```json
{
  "collector_name": "WindowsProcessCollector",
  "collector_version": "0.1.0",
  "count": 1,
  "processes": [
    {
      "pid": 1024,
      "ppid": 512,
      "name": "cargo.exe",
      "path": "C:\\Users\\...\\.cargo\\bin\\cargo.exe",
      "memory_bytes": 0
    }
  ]
}
```

---

## 3. Network Collector (`WindowsNetworkCollector`)
* **Category**: `"Network"`
* **Collector Version**: `0.1.0`
* **Win32 APIs**:
  * `GetExtendedTcpTable` (`TCP_TABLE_OWNER_PID_ALL`, `AF_INET`): Enumeration of IPv4 TCP endpoints mapped to owning process IDs.
* **Resilience & Timing Handling**:
  * **Dynamic Buffer Resizing**: Retries up to 4 times when `ERROR_INSUFFICIENT_BUFFER` occurs due to rapid socket changes.
* **Supported Modes**:
  * `"connections"`: Returns all active non-listening TCP endpoints.
  * `"listeners"`: Filters strictly for listening TCP sockets (`state == "LISTEN"`).
* **Payload Schema**:
```json
{
  "mode": "connections",
  "count": 1,
  "records": [
    {
      "local_addr": "127.0.0.1",
      "local_port": 8080,
      "remote_addr": "127.0.0.1",
      "remote_port": 51234,
      "state": "ESTABLISHED",
      "pid": 2048,
      "protocol": "TCP"
    }
  ]
}
```

---

## Stability Verification
Windows collectors pass a 10-consecutive-iteration regression test suite:
* `test_windows_system_consecutive_10_runs`: Verified memory and uptime integrity.
* `test_windows_process_consecutive_10_runs`: Verified stability against active process churn and access denial.
* `test_windows_network_consecutive_10_runs`: Verified buffer stability across rapid TCP table scans.