//! JOCKY Forensic Runtime (Layer 3)
//!
//! Responsible for:
//! 1. Receiving a ForensicIrDocument (real IR from compiler crate)
//! 2. Locally re-validating the IR using IrValidator before any execution
//! 3. Processing instructions in sequence order
//! 4. Passing operations to Dispatcher -> Collectors
//! 5. Returning collected RawEvidence packages

use std::collections::HashMap;
use thiserror::Error;

use crate::dispatcher::{Dispatcher, DispatcherError};
use crate::evidence::RawEvidence;
use compiler::ir::ForensicIrDocument;
use compiler::validator::{IrValidationError, IrValidator};

/// Errors emitted by the JOCKY Runtime.
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Local IR validation failed before dispatch: {0}")]
    ValidationFailed(#[from] IrValidationError),

    #[error("Instruction dispatch error at sequence {sequence} ('{op}'): {source}")]
    DispatchFailed {
        sequence: usize,
        op: String,
        #[source]
        source: DispatcherError,
    },
}

/// The JOCKY Execution Runtime
pub struct Runtime {
    pub dispatcher: Dispatcher,
}

impl Runtime {
    /// Create a new Runtime with default Dispatcher.
    pub fn new() -> Self {
        Self {
            dispatcher: Dispatcher::new(),
        }
    }

    /// Create a Runtime bound to a specific machine identity.
    pub fn with_machine_id(machine_id: impl Into<String>) -> Self {
        Self {
            dispatcher: Dispatcher::with_machine_id(machine_id),
        }
    }

    /// Create a Runtime with a specific Dispatcher.
    pub fn with_dispatcher(dispatcher: Dispatcher) -> Self {
        Self { dispatcher }
    }

    /// Execute an investigation across multiple target machines in isolation.
    ///
    /// Validation occurs once upfront via IrValidator::validate(doc).
    /// Each target machine receives an isolated Runtime instance with its own Dispatcher
    /// and runtime_memory state, preserving strict machine boundary isolation.
    pub fn execute_fan_out(
        doc: &ForensicIrDocument,
        target_machines: &[&str],
    ) -> Result<HashMap<String, Vec<RawEvidence>>, RuntimeError> {
        IrValidator::validate(doc)?;

        let mut results_by_machine = HashMap::new();
        for &machine_id in target_machines {
            let mut machine_runtime = Runtime::with_machine_id(machine_id);
            let evidence = machine_runtime.execute(doc)?;
            results_by_machine.insert(machine_id.to_string(), evidence);
        }
        Ok(results_by_machine)
    }

    /// Execute a Forensic IR Document end-to-end:
    /// - Re-validates the IR locally against the schema and allowlist.
    /// - Dispatches approved instructions to the OS collector.
    /// - Returns a vector of all collected RawEvidence objects.
    pub fn execute(&mut self, doc: &ForensicIrDocument) -> Result<Vec<RawEvidence>, RuntimeError> {
        // Step 1: Mandatory local IR validation checkpoint
        println!("Runtime: validating IR...");
        IrValidator::validate(doc)?;

        let mut results = Vec::new();

        // Step 2: Sequential instruction dispatch
        println!("Runtime: starting dispatcher loop...");
        for inst in &doc.instructions {
            println!("Runtime: dispatching instruction {} - {}", inst.sequence, inst.op);
            match self.dispatcher.dispatch_instruction(inst) {
                Ok(Some(evidence)) => {
                    results.push(evidence);
                }
                Ok(None) => {
                    // Container boundaries (INVESTIGATION_BEGIN, INVESTIGATION_END)
                }
                Err(e) => {
                    return Err(RuntimeError::DispatchFailed {
                        sequence: inst.sequence,
                        op: inst.op.clone(),
                        source: e,
                    });
                }
            }
        }
        println!("Runtime: dispatcher loop completed. Evidence count: {}", results.len());
        Ok(results)
    }

    /// Convenience execution from a raw JSON string.
    pub fn execute_json(&mut self, raw_json: &str) -> Result<Vec<RawEvidence>, RuntimeError> {
        let doc = IrValidator::validate_json(raw_json)?;
        self.execute(&doc)
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compiler::ast::*;
    use compiler::ir::IrGenerator;
    use serde_json::json;

    fn dummy_span() -> Span {
        Span {
    line: 1,
    column: 1,
}
    }

    #[test]
    fn test_day6_real_collector_system_info_e2e() {
        // Build valid AST for System.info() investigation
        let inv = Investigation {
            name: StringLiteral {
                value: "Day 6 System Info Triage".to_string(),
                span: dummy_span(),
            },
            body: vec![
                Statement::Assignment(Assignment {
                    target: Identifier { name: "sys".to_string(), span: dummy_span() },
                    type_annotation: None,
                    value: Expr::Call(Call {
                        object: Box::new(Expr::Identifier(Identifier { name: "System".to_string(), span: dummy_span() })),
                        function: Identifier { name: "info".to_string(), span: dummy_span() },
                        arguments: vec![],
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                }),
            ],
            span: dummy_span(),
        };

        // 1. Generate real Forensic IR
        let generator = IrGenerator::new("inv-day6-test").with_target_scope("WIN-CLIENT-07");
        let ir_doc = generator.generate_from_investigation(&inv).expect("IR generation must succeed");

        // Verify generated IR structure
        assert_eq!(ir_doc.ir_version, "v0.1.0");
        assert_eq!(ir_doc.instructions.len(), 3);
        assert_eq!(ir_doc.instructions[0].op, "INVESTIGATION_BEGIN");
        assert_eq!(ir_doc.instructions[1].op, "COLLECT_SYSTEM_INFO");
        assert_eq!(ir_doc.instructions[2].op, "INVESTIGATION_END");

        // 2. Initialize Runtime and execute real IR
        let mut runtime = Runtime::new();
        let results = runtime.execute(&ir_doc).expect("Runtime execution must succeed with real collector");

        // 3. Verify that real Windows system collector data was returned
        assert_eq!(results.len(), 1);
        let evidence = &results[0];
        assert_eq!(evidence.category, "System");

        let payload = &evidence.payload;
        assert_eq!(payload["collector_name"], "WindowsSystemCollector");
        assert_eq!(payload["os_version"], "Windows");
        assert!(payload["os_release"].as_str().unwrap().contains("Windows NT"));

        // Prove that REAL Windows system information was collected, not hardcoded mocks
        let computer_name = payload["computer_name"].as_str().expect("computer_name must be a string");
        assert_ne!(computer_name, "UNKNOWN", "Real computer name must be retrieved from Windows OS");
        assert!(!computer_name.is_empty());

        let total_mem = payload["total_physical_memory"].as_u64().expect("total_physical_memory must be a number");
        assert!(total_mem > 0, "Real physical memory must be greater than zero bytes");

        let uptime = payload["uptime_seconds"].as_u64().expect("uptime_seconds must be a number");
        assert!(uptime > 0, "System uptime must be positive");

        println!("=== DAY 6 REAL WINDOWS SYSTEM COLLECTOR VERIFIED ===");
        println!("Computer Name : {}", computer_name);
        println!("Total Memory  : {} bytes ({:.2} GB)", total_mem, total_mem as f64 / (1024.0 * 1024.0 * 1024.0));
        println!("System Uptime : {} seconds", uptime);
        println!("Architecture  : {}", payload["architecture"]);
    }

    #[test]
    fn test_runtime_rejects_invalid_ir_version_before_dispatch() {
        // Construct IR with wrong version
        let bad_ir_json = serde_json::to_string(&json!({
            "ir_version": "v0.9.9",
            "investigation_id": "inv-bad-ver",
            "investigation_name": "Bad Version Investigation",
            "target_scope": "ALL",
            "instructions": [
                {
                    "sequence": 1,
                    "op": "INVESTIGATION_BEGIN",
                    "params": {
                        "investigation_id": "inv-bad-ver",
                        "investigation_name": "Bad Version Investigation",
                        "timestamp_utc": "2026-09-24T13:46:00Z"
                    }
                },
                {
                    "sequence": 2,
                    "op": "COLLECT_SYSTEM_INFO",
                    "params": { "output_variable": "sys" }
                },
                {
                    "sequence": 3,
                    "op": "INVESTIGATION_END",
                    "params": { "investigation_id": "inv-bad-ver", "status": "SUCCESS" }
                }
            ]
        })).unwrap();

        let mut runtime = Runtime::new();
        let res = runtime.execute_json(&bad_ir_json);
        assert!(res.is_err(), "Runtime must reject unsupported IR version");
        match res.unwrap_err() {
            RuntimeError::ValidationFailed(IrValidationError::UnsupportedVersion { version, .. }) => {
                assert_eq!(version, "v0.9.9");
            }
            other => panic!("Expected ValidationFailed(UnsupportedVersion), got: {:?}", other),
        }
    }

    #[test]
    fn test_runtime_rejects_unsupported_operation_before_dispatch() {
        // Construct IR with unapproved opcode
        let bad_ir_json = serde_json::to_string(&json!({
            "ir_version": "v0.1.0",
            "investigation_id": "inv-bad-op",
            "investigation_name": "Bad Op Investigation",
            "target_scope": "ALL",
            "instructions": [
                {
                    "sequence": 1,
                    "op": "INVESTIGATION_BEGIN",
                    "params": {
                        "investigation_id": "inv-bad-op",
                        "investigation_name": "Bad Op Investigation",
                        "timestamp_utc": "2026-09-24T13:46:00Z"
                    }
                },
                {
                    "sequence": 2,
                    "op": "EXECUTE_ARBITRARY_SHELL",
                    "params": { "command": "rm -rf /" }
                },
                {
                    "sequence": 3,
                    "op": "INVESTIGATION_END",
                    "params": { "investigation_id": "inv-bad-op", "status": "SUCCESS" }
                }
            ]
        })).unwrap();

        let mut runtime = Runtime::new();
        let res = runtime.execute_json(&bad_ir_json);
        match res.unwrap_err() {
            RuntimeError::ValidationFailed(IrValidationError::UnsupportedOperation { op, .. }) => {
                assert_eq!(op, "EXECUTE_ARBITRARY_SHELL");
            }
            other => panic!("Expected ValidationFailed(UnsupportedOperation), got: {:?}", other),
        }
    }

    #[test]
    fn test_runtime_executes_all_5_operations_e2e() {
        let full_ir_json = serde_json::to_string(&json!({
            "ir_version": "v0.1.0",
            "investigation_id": "inv-day7-full",
            "investigation_name": "Full 5-Operation Triage",
            "target_scope": "WIN-CLIENT-07",
            "instructions": [
                {
                    "sequence": 1,
                    "op": "INVESTIGATION_BEGIN",
                    "params": {
                        "investigation_id": "inv-day7-full",
                        "investigation_name": "Full 5-Operation Triage",
                        "timestamp_utc": "2026-09-24T13:46:00Z"
                    }
                },
                {
                    "sequence": 2,
                    "op": "COLLECT_SYSTEM_INFO",
                    "params": { "output_variable": "sys" }
                },
                {
                    "sequence": 3,
                    "op": "COLLECT_PROCESSES",
                    "params": { "output_variable": "procs" }
                },
                {
                    "sequence": 4,
                    "op": "FILTER_PROCESSES",
                    "params": {
                        "source_variable": "procs",
                        "output_variable": "flagged",
                        "predicate": {
                            "type": "Comparison",
                            "op": "!=",
                            "field": "name",
                            "literal": { "type": "String", "value": "" }
                        }
                    }
                },
                {
                    "sequence": 5,
                    "op": "COLLECT_NETWORK_CONNECTIONS",
                    "params": { "output_variable": "conns" }
                },
                {
                    "sequence": 6,
                    "op": "COLLECT_NETWORK_LISTENERS",
                    "params": { "output_variable": "listeners" }
                },
                {
                    "sequence": 7,
                    "op": "INVESTIGATION_END",
                    "params": {
                        "investigation_id": "inv-day7-full",
                        "status": "SUCCESS"
                    }
                }
            ]
        })).unwrap();

        let mut runtime = Runtime::new();
        let results = runtime.execute_json(&full_ir_json).expect("Full 5-operation IR execution must succeed");

        assert_eq!(results.len(), 5);
        assert_eq!(results[0].category, "System");
        assert_eq!(results[1].category, "Process");
        assert_eq!(results[2].category, "Process");
        assert_eq!(results[3].category, "Network");
        assert_eq!(results[4].category, "Network");

        assert_eq!(results[3].payload["mode"], "connections");
        assert_eq!(results[4].payload["mode"], "listeners");

        println!("=== DAY 7 ALL 5 OPERATIONS VERIFIED END-TO-END IN RUNTIME ===");
        println!("1. System Info : {}", results[0].payload["computer_name"]);
        println!("2. Procs Count : {}", results[1].payload["count"]);
        println!("3. Filter Count: {}", results[2].payload["count"]);
        println!("4. Conn Count  : {}", results[3].payload["count"]);
        println!("5. Listen Count: {}", results[4].payload["count"]);
    }

    #[test]
    fn test_multi_instruction_collect_and_filter() {
        let ir = serde_json::to_string(&json!({
            "ir_version": "v0.1.0",
            "investigation_id": "inv-cf-01",
            "investigation_name": "Collect and Filter",
            "instructions": [
                { "sequence": 1, "op": "INVESTIGATION_BEGIN", "params": { "investigation_id": "inv-cf-01", "investigation_name": "Collect and Filter", "timestamp_utc": "2026-09-24T13:46:00Z" } },
                { "sequence": 2, "op": "COLLECT_PROCESSES", "params": { "output_variable": "procs" } },
                { "sequence": 3, "op": "FILTER_PROCESSES", "params": {
                    "source_variable": "procs",
                    "output_variable": "filtered",
                    "predicate": { "type": "Comparison", "op": ">=", "field": "pid", "literal": { "type": "Integer", "value": 0 } }
                }},
                { "sequence": 4, "op": "INVESTIGATION_END", "params": { "investigation_id": "inv-cf-01", "status": "SUCCESS" } }
            ]
        })).unwrap();

        let mut runtime = Runtime::new();
        let res = runtime.execute_json(&ir).expect("Collect and Filter sequence failed");
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].category, "Process");
        assert_eq!(res[1].category, "Process");
        assert!(runtime.dispatcher.runtime_memory.contains_key("procs"));
        assert!(runtime.dispatcher.runtime_memory.contains_key("filtered"));
    }

    #[test]
    fn test_multi_instruction_collect_filter_network_connections() {
        let ir = serde_json::to_string(&json!({
            "ir_version": "v0.1.0",
            "investigation_id": "inv-cfc-01",
            "investigation_name": "Collect Filter Conns",
            "instructions": [
                { "sequence": 1, "op": "INVESTIGATION_BEGIN", "params": { "investigation_id": "inv-cfc-01", "investigation_name": "Collect Filter Conns", "timestamp_utc": "2026-09-24T13:46:00Z" } },
                { "sequence": 2, "op": "COLLECT_PROCESSES", "params": { "output_variable": "procs" } },
                { "sequence": 3, "op": "FILTER_PROCESSES", "params": {
                    "source_variable": "procs",
                    "output_variable": "filtered",
                    "predicate": { "type": "Comparison", "op": ">=", "field": "pid", "literal": { "type": "Integer", "value": 0 } }
                }},
                { "sequence": 4, "op": "COLLECT_NETWORK_CONNECTIONS", "params": { "output_variable": "conns" } },
                { "sequence": 5, "op": "INVESTIGATION_END", "params": { "investigation_id": "inv-cfc-01", "status": "SUCCESS" } }
            ]
        })).unwrap();

        let mut runtime = Runtime::new();
        let res = runtime.execute_json(&ir).expect("3-stage sequence failed");
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].category, "Process");
        assert_eq!(res[1].category, "Process");
        assert_eq!(res[2].category, "Network");
        assert_eq!(res[2].payload["mode"], "connections");
    }

    #[test]
    fn test_multi_instruction_collect_filter_connections_listeners() {
        let ir = serde_json::to_string(&json!({
            "ir_version": "v0.1.0",
            "investigation_id": "inv-cfcl-01",
            "investigation_name": "Collect Filter Conns Listeners",
            "instructions": [
                { "sequence": 1, "op": "INVESTIGATION_BEGIN", "params": { "investigation_id": "inv-cfcl-01", "investigation_name": "Collect Filter Conns Listeners", "timestamp_utc": "2026-09-24T13:46:00Z" } },
                { "sequence": 2, "op": "COLLECT_PROCESSES", "params": { "output_variable": "procs" } },
                { "sequence": 3, "op": "FILTER_PROCESSES", "params": {
                    "source_variable": "procs",
                    "output_variable": "filtered",
                    "predicate": { "type": "Comparison", "op": ">=", "field": "pid", "literal": { "type": "Integer", "value": 0 } }
                }},
                { "sequence": 4, "op": "COLLECT_NETWORK_CONNECTIONS", "params": { "output_variable": "conns" } },
                { "sequence": 5, "op": "COLLECT_NETWORK_LISTENERS", "params": { "output_variable": "listeners" } },
                { "sequence": 6, "op": "INVESTIGATION_END", "params": { "investigation_id": "inv-cfcl-01", "status": "SUCCESS" } }
            ]
        })).unwrap();

        let mut runtime = Runtime::new();
        let res = runtime.execute_json(&ir).expect("4-stage sequence failed");
        assert_eq!(res.len(), 4);
        assert_eq!(res[0].category, "Process");
        assert_eq!(res[1].category, "Process");
        assert_eq!(res[2].category, "Network");
        assert_eq!(res[3].category, "Network");
        assert_eq!(res[2].payload["mode"], "connections");
        assert_eq!(res[3].payload["mode"], "listeners");
    }

    #[test]
    fn test_runtime_memory_isolated_between_investigations() {
        let ir_inv1 = serde_json::to_string(&json!({
            "ir_version": "v0.1.0",
            "investigation_id": "inv-01",
            "investigation_name": "Inv 1",
            "instructions": [
                { "sequence": 1, "op": "INVESTIGATION_BEGIN", "params": { "investigation_id": "inv-01", "investigation_name": "Inv 1", "timestamp_utc": "2026-09-24T13:46:00Z" } },
                { "sequence": 2, "op": "COLLECT_SYSTEM_INFO", "params": { "output_variable": "secret_var_1" } },
                { "sequence": 3, "op": "INVESTIGATION_END", "params": { "investigation_id": "inv-01", "status": "SUCCESS" } }
            ]
        })).unwrap();

        let ir_inv2 = serde_json::to_string(&json!({
            "ir_version": "v0.1.0",
            "investigation_id": "inv-02",
            "investigation_name": "Inv 2",
            "instructions": [
                { "sequence": 1, "op": "INVESTIGATION_BEGIN", "params": { "investigation_id": "inv-02", "investigation_name": "Inv 2", "timestamp_utc": "2026-09-24T13:46:00Z" } },
                { "sequence": 2, "op": "COLLECT_SYSTEM_INFO", "params": { "output_variable": "var_2" } },
                { "sequence": 3, "op": "INVESTIGATION_END", "params": { "investigation_id": "inv-02", "status": "SUCCESS" } }
            ]
        })).unwrap();

        let mut runtime = Runtime::new();
        // Run first investigation
        runtime.execute_json(&ir_inv1).unwrap();
        assert!(runtime.dispatcher.runtime_memory.contains_key("secret_var_1"));

        // Run second investigation on the same runtime
        runtime.execute_json(&ir_inv2).unwrap();
        // Memory from investigation 1 must NOT be present in investigation 2!
        assert!(!runtime.dispatcher.runtime_memory.contains_key("secret_var_1"),
            "Investigation 1 memory must NOT leak into Investigation 2");
        assert!(runtime.dispatcher.runtime_memory.contains_key("var_2"));
    }

    #[test]
    fn test_multi_machine_fan_out_isolation_and_identity() {
        let doc_json = serde_json::to_string(&json!({
            "ir_version": "v0.1.0",
            "investigation_id": "inv-fan-01",
            "investigation_name": "Multi-Machine Triage",
            "target_scope": "WIN-NODE-A, WIN-NODE-B",
            "instructions": [
                { "sequence": 1, "op": "INVESTIGATION_BEGIN", "params": { "investigation_id": "inv-fan-01", "investigation_name": "Multi-Machine Triage", "timestamp_utc": "2026-09-24T13:46:00Z" } },
                { "sequence": 2, "op": "COLLECT_SYSTEM_INFO", "params": { "output_variable": "sys" } },
                { "sequence": 3, "op": "COLLECT_NETWORK_LISTENERS", "params": { "output_variable": "listeners" } },
                { "sequence": 4, "op": "INVESTIGATION_END", "params": { "investigation_id": "inv-fan-01", "status": "SUCCESS" } }
            ]
        })).unwrap();

        let doc: ForensicIrDocument = serde_json::from_str(&doc_json).unwrap();
        let targets = ["WIN-NODE-A", "WIN-NODE-B"];

        let results = Runtime::execute_fan_out(&doc, &targets).expect("Fan-out execution must succeed");

        assert_eq!(results.len(), 2);
        assert!(results.contains_key("WIN-NODE-A"));
        assert!(results.contains_key("WIN-NODE-B"));

        let ev_a = &results["WIN-NODE-A"];
        let ev_b = &results["WIN-NODE-B"];

        assert_eq!(ev_a.len(), 2);
        assert_eq!(ev_b.len(), 2);

        // Verify machine identity attribution in evidence payload
        for ev in ev_a {
            assert_eq!(ev.payload["machine_id"], "WIN-NODE-A", "Evidence must be attributed to WIN-NODE-A");
        }
        for ev in ev_b {
            assert_eq!(ev.payload["machine_id"], "WIN-NODE-B", "Evidence must be attributed to WIN-NODE-B");
        }

        println!("=== MULTI-MACHINE FAN-OUT VERIFIED (WIN-NODE-A + WIN-NODE-B) ===");
        println!("WIN-NODE-A evidence count: {}", ev_a.len());
        println!("WIN-NODE-B evidence count: {}", ev_b.len());
    }

    #[test]
    fn test_runtime_handles_missing_source_variable_safely() {
        let ir = serde_json::to_string(&json!({
            "ir_version": "v0.1.0",
            "investigation_id": "inv-missing-var",
            "investigation_name": "Missing Variable Test",
            "instructions": [
                { "sequence": 1, "op": "INVESTIGATION_BEGIN", "params": { "investigation_id": "inv-missing-var", "investigation_name": "Missing Variable Test", "timestamp_utc": "2026-09-24T13:46:00Z" } },
                { "sequence": 2, "op": "FILTER_PROCESSES", "params": {
                    "source_variable": "non_existent_procs",
                    "output_variable": "filtered",
                    "predicate": { "type": "Comparison", "op": "==", "field": "name", "literal": { "type": "String", "value": "test.exe" } }
                }},
                { "sequence": 3, "op": "INVESTIGATION_END", "params": { "investigation_id": "inv-missing-var", "status": "SUCCESS" } }
            ]
        })).unwrap();

        let mut runtime = Runtime::new();
        let res = runtime.execute_json(&ir);
        assert!(res.is_err(), "Runtime must fail when source_variable does not exist");
        match res.unwrap_err() {
            RuntimeError::DispatchFailed { sequence, op, source } => {
                assert_eq!(sequence, 2);
                assert_eq!(op, "FILTER_PROCESSES");
                match source {
                    DispatcherError::MissingSourceVariable(var) => {
                        assert_eq!(var, "non_existent_procs");
                    }
                    other => panic!("Expected MissingSourceVariable, got: {:?}", other),
                }
            }
            other => panic!("Expected DispatchFailed error, got: {:?}", other),
        }
    }

    #[test]
    fn test_runtime_failure_in_middle_instruction_aborts_without_success() {
        let ir = serde_json::to_string(&json!({
            "ir_version": "v0.1.0",
            "investigation_id": "inv-mid-fail",
            "investigation_name": "Middle Failure Test",
            "instructions": [
                { "sequence": 1, "op": "INVESTIGATION_BEGIN", "params": { "investigation_id": "inv-mid-fail", "investigation_name": "Middle Failure Test", "timestamp_utc": "2026-09-24T13:46:00Z" } },
                { "sequence": 2, "op": "COLLECT_SYSTEM_INFO", "params": { "output_variable": "sys" } },
                { "sequence": 3, "op": "FILTER_PROCESSES", "params": {
                    "source_variable": "missing_procs",
                    "output_variable": "filtered",
                    "predicate": { "type": "Comparison", "op": "==", "field": "name", "literal": { "type": "String", "value": "test.exe" } }
                }},
                { "sequence": 4, "op": "COLLECT_NETWORK_CONNECTIONS", "params": { "output_variable": "conns" } },
                { "sequence": 5, "op": "INVESTIGATION_END", "params": { "investigation_id": "inv-mid-fail", "status": "SUCCESS" } }
            ]
        })).unwrap();

        let mut runtime = Runtime::new();
        let res = runtime.execute_json(&ir);

        // 1. Must fail, not succeed silently
        assert!(res.is_err(), "Middle instruction failure must abort and return Err");
        match res.unwrap_err() {
            RuntimeError::DispatchFailed { sequence, op, .. } => {
                assert_eq!(sequence, 3);
                assert_eq!(op, "FILTER_PROCESSES");
            }
            other => panic!("Expected DispatchFailed, got: {:?}", other),
        }

        // 2. Subsequent instruction (sequence 4: COLLECT_NETWORK_CONNECTIONS) must NOT have run
        assert!(!runtime.dispatcher.runtime_memory.contains_key("conns"),
            "Subsequent instruction must not execute after middle instruction failure");

        // 3. Prior instruction (sequence 2: COLLECT_SYSTEM_INFO) did run before failure
        assert!(runtime.dispatcher.runtime_memory.contains_key("sys"),
            "Prior instruction result should be in runtime memory prior to abort");
    }
}


