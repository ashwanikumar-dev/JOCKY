//! JOCKY Forensic Intermediate Representation (Forensic IR)
//!
//! Authoritative M3 implementation for Forensic IR v0.1.0.
//! Maps semantically validated JOCKY AST into a closed, allowlisted,
//! versioned, and JSON-serializable execution plan containing exactly
//! the 9 MVP forensic instructions.
//!
//! Specifications:
//! - JOCKY Grammar Specification v0.1 (Section 11: Grammar-to-IR Considerations)
//! - JOCKY Engineering Handbook (Section 15: Forensic IR, Section 37: Security)
//! - schemas/ir_examples.json (Authoritative frozen Day 1 IR contract)

use std::fmt;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

use crate::ast::{
    ComparisonOp, Expr, Investigation, Literal, LogicalBinaryOp, LogicalUnaryOp, Program,
    Statement,
};

/// The frozen IR version for JOCKY MVP.
pub const IR_VERSION: &str = "v0.1.0";

/// The authoritative 9-instruction allowlist for MVP Forensic IR.
pub const ALLOWED_OPERATIONS: &[&str] = &[
    "INVESTIGATION_BEGIN",
    "COLLECT_SYSTEM_INFO",
    "COLLECT_PROCESSES",
    "FILTER_PROCESSES",
    "COLLECT_NETWORK_CONNECTIONS",
    "COLLECT_NETWORK_LISTENERS",
    "PRESERVE_EVIDENCE",
    "VERIFY_EVIDENCE",
    "INVESTIGATION_END",
];

/// The 9 MVP Forensic IR Operation Opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IrOp {
    #[serde(rename = "INVESTIGATION_BEGIN")]
    InvestigationBegin,
    #[serde(rename = "COLLECT_SYSTEM_INFO")]
    CollectSystemInfo,
    #[serde(rename = "COLLECT_PROCESSES")]
    CollectProcesses,
    #[serde(rename = "FILTER_PROCESSES")]
    FilterProcesses,
    #[serde(rename = "COLLECT_NETWORK_CONNECTIONS")]
    CollectNetworkConnections,
    #[serde(rename = "COLLECT_NETWORK_LISTENERS")]
    CollectNetworkListeners,
    #[serde(rename = "PRESERVE_EVIDENCE")]
    PreserveEvidence,
    #[serde(rename = "VERIFY_EVIDENCE")]
    VerifyEvidence,
    #[serde(rename = "INVESTIGATION_END")]
    InvestigationEnd,
}

impl IrOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            IrOp::InvestigationBegin        => "INVESTIGATION_BEGIN",
            IrOp::CollectSystemInfo         => "COLLECT_SYSTEM_INFO",
            IrOp::CollectProcesses          => "COLLECT_PROCESSES",
            IrOp::FilterProcesses           => "FILTER_PROCESSES",
            IrOp::CollectNetworkConnections => "COLLECT_NETWORK_CONNECTIONS",
            IrOp::CollectNetworkListeners   => "COLLECT_NETWORK_LISTENERS",
            IrOp::PreserveEvidence          => "PRESERVE_EVIDENCE",
            IrOp::VerifyEvidence            => "VERIFY_EVIDENCE",
            IrOp::InvestigationEnd          => "INVESTIGATION_END",
        }
    }
}

impl fmt::Display for IrOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A complete Forensic IR Document representing an investigation execution plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForensicIrDocument {
    pub ir_version: String,
    pub investigation_id: String,
    pub investigation_name: String,
    #[serde(default = "default_target_scope")]
    pub target_scope: String,
    pub instructions: Vec<IrInstruction>,
}

fn default_target_scope() -> String {
    "ALL".to_string()
}

/// A single lowered IR instruction matching schemas/ir_examples.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrInstruction {
    pub sequence: usize,
    pub op: String,
    pub params: Map<String, Value>,
}

/// Structured filter predicate tree matching the JSON schema in schemas/ir_examples.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Predicate {
    LogicalAnd {
        left: Box<Predicate>,
        right: Box<Predicate>,
    },
    LogicalOr {
        left: Box<Predicate>,
        right: Box<Predicate>,
    },
    LogicalNot {
        operand: Box<Predicate>,
    },
    Comparison {
        op: String,
        field: String,
        literal: PredicateLiteral,
    },
}

/// Structured literal value used inside comparison predicates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PredicateLiteral {
    String {
        value: String,
    },
    Integer {
        value: i64,
    },
    Float {
        value: f64,
    },
    Boolean {
        value: bool,
    },
    ByteSize {
        value_bytes: u64,
        raw_unit: String,
    },
}

/// Errors that can occur during AST → IR generation.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IrGenError {
    #[error("Empty investigation body: cannot generate IR for investigation '{title}'")]
    EmptyInvestigation { title: String },

    #[error("Unsupported AST expression for IR lowering: '{reason}'")]
    UnsupportedExpression { reason: String },

    #[error("Predicate conversion error: '{reason}'")]
    PredicateError { reason: String },
}

/// AST → Forensic IR Generator
pub struct IrGenerator {
    investigation_id: String,
    target_scope: String,
    timestamp_utc: String,
}

impl IrGenerator {
    /// Create a new generator with an explicit investigation ID.
    pub fn new(investigation_id: impl Into<String>) -> Self {
        Self {
            investigation_id: investigation_id.into(),
            target_scope: "ALL".to_string(),
            timestamp_utc: "2026-09-24T13:46:00Z".to_string(),
        }
    }

    /// Set a custom target scope (e.g., machine identifier or tag).
    pub fn with_target_scope(mut self, scope: impl Into<String>) -> Self {
        self.target_scope = scope.into();
        self
    }

    /// Set a custom ISO-8601 UTC timestamp.
    pub fn with_timestamp_utc(mut self, ts: impl Into<String>) -> Self {
        self.timestamp_utc = ts.into();
        self
    }

    /// Generate a ForensicIrDocument from a top-level Program AST.
    pub fn generate(&self, program: &Program) -> Result<ForensicIrDocument, IrGenError> {
        self.generate_from_investigation(&program.investigation)
    }

    /// Generate a ForensicIrDocument from an Investigation AST node.
    pub fn generate_from_investigation(&self, inv: &Investigation) -> Result<ForensicIrDocument, IrGenError> {
        let inv_name = inv.name.value.clone();
        if inv.body.is_empty() {
            return Err(IrGenError::EmptyInvestigation { title: inv_name });
        }

        let mut instructions = Vec::new();
        let mut sequence = 1;

        // 1. INVESTIGATION_BEGIN
        let mut begin_params = Map::new();
        begin_params.insert("investigation_id".to_string(), Value::String(self.investigation_id.clone()));
        begin_params.insert("investigation_name".to_string(), Value::String(inv_name.clone()));
        begin_params.insert("timestamp_utc".to_string(), Value::String(self.timestamp_utc.clone()));

        instructions.push(IrInstruction {
            sequence,
            op: IrOp::InvestigationBegin.as_str().to_string(),
            params: begin_params,
        });

        // 2. Body statements lowering
        for stmt in &inv.body {
            self.lower_statement(stmt, &mut instructions, &mut sequence)?;
        }

        // 3. INVESTIGATION_END
        sequence += 1;
        let mut end_params = Map::new();
        end_params.insert("investigation_id".to_string(), Value::String(self.investigation_id.clone()));
        end_params.insert("status".to_string(), Value::String("SUCCESS".to_string()));

        instructions.push(IrInstruction {
            sequence,
            op: IrOp::InvestigationEnd.as_str().to_string(),
            params: end_params,
        });

        Ok(ForensicIrDocument {
            ir_version: IR_VERSION.to_string(),
            investigation_id: self.investigation_id.clone(),
            investigation_name: inv_name,
            target_scope: self.target_scope.clone(),
            instructions,
        })
    }

    fn lower_statement(
        &self,
        stmt: &Statement,
        instructions: &mut Vec<IrInstruction>,
        sequence: &mut usize,
    ) -> Result<(), IrGenError> {
        match stmt {
            Statement::Assignment(assignment) => {
                let target_var = assignment.target.name.clone();
                self.lower_assignment_value(&assignment.value, &target_var, instructions, sequence)?;
            }
            Statement::ExpressionStatement(expr) => {
                self.lower_standalone_expression(expr, instructions, sequence)?;
            }
        }
        Ok(())
    }

    fn lower_assignment_value(
        &self,
        value_expr: &Expr,
        target_var: &str,
        instructions: &mut Vec<IrInstruction>,
        sequence: &mut usize,
    ) -> Result<(), IrGenError> {
        match value_expr {
            Expr::Call(call) => {
                if let Expr::Identifier(ref mod_id) = *call.object {
                    match (mod_id.name.as_str(), call.function.name.as_str()) {
                        ("System", "info") => {
                            *sequence += 1;
                            let mut params = Map::new();
                            params.insert("output_variable".to_string(), Value::String(target_var.to_string()));
                            instructions.push(IrInstruction {
                                sequence: *sequence,
                                op: IrOp::CollectSystemInfo.as_str().to_string(),
                                params,
                            });
                            return Ok(());
                        }
                        ("Process", "collect") => {
                            *sequence += 1;
                            let mut params = Map::new();
                            params.insert("output_variable".to_string(), Value::String(target_var.to_string()));
                            instructions.push(IrInstruction {
                                sequence: *sequence,
                                op: IrOp::CollectProcesses.as_str().to_string(),
                                params,
                            });
                            return Ok(());
                        }
                        ("Network", "connections") => {
                            *sequence += 1;
                            let mut params = Map::new();
                            params.insert("output_variable".to_string(), Value::String(target_var.to_string()));
                            instructions.push(IrInstruction {
                                sequence: *sequence,
                                op: IrOp::CollectNetworkConnections.as_str().to_string(),
                                params,
                            });
                            return Ok(());
                        }
                        ("Network", "listeners") => {
                            *sequence += 1;
                            let mut params = Map::new();
                            params.insert("output_variable".to_string(), Value::String(target_var.to_string()));
                            instructions.push(IrInstruction {
                                sequence: *sequence,
                                op: IrOp::CollectNetworkListeners.as_str().to_string(),
                                params,
                            });
                            return Ok(());
                        }
                        ("Evidence", "preserve") => {
                            *sequence += 1;
                            let source_var = self.extract_identifier_arg(&call.arguments, 0)?;
                            let mut params = Map::new();
                            params.insert("source_variable".to_string(), Value::String(source_var));
                            params.insert("evidence_tag".to_string(), Value::String(target_var.to_string()));
                            instructions.push(IrInstruction {
                                sequence: *sequence,
                                op: IrOp::PreserveEvidence.as_str().to_string(),
                                params,
                            });
                            return Ok(());
                        }
                        ("Evidence", "verify") => {
                            *sequence += 1;
                            let evidence_var = self.extract_identifier_arg(&call.arguments, 0)?;
                            let mut params = Map::new();
                            params.insert("evidence_variable".to_string(), Value::String(evidence_var));
                            instructions.push(IrInstruction {
                                sequence: *sequence,
                                op: IrOp::VerifyEvidence.as_str().to_string(),
                                params,
                            });
                            return Ok(());
                        }
                        _ => {}
                    }
                }

                // Method call on variable, e.g. procs.filter(...)
                if call.function.name == "filter" {
                    let source_var = match &*call.object {
                        Expr::Identifier(ident) => ident.name.clone(),
                        _ => return Err(IrGenError::UnsupportedExpression {
                            reason: "Complex filter receiver not an identifier".to_string(),
                        }),
                    };
                    let predicate = self.lower_predicate(&call.arguments[0])?;
                    *sequence += 1;
                    let mut params = Map::new();
                    params.insert("source_variable".to_string(), Value::String(source_var));
                    params.insert("output_variable".to_string(), Value::String(target_var.to_string()));
                    params.insert("predicate".to_string(), serde_json::to_value(predicate).map_err(|e| {
                        IrGenError::PredicateError { reason: e.to_string() }
                    })?);

                    instructions.push(IrInstruction {
                        sequence: *sequence,
                        op: IrOp::FilterProcesses.as_str().to_string(),
                        params,
                    });
                    return Ok(());
                }

                Err(IrGenError::UnsupportedExpression {
                    reason: format!("Unknown call shape in assignment to '{}'", target_var),
                })
            }

            Expr::Filter(filter) => {
                let source_var = match &*filter.source {
                    Expr::Identifier(ident) => ident.name.clone(),
                    Expr::Call(chained) => {
                        // Chained: Process.collect().filter(...)
                        // Emit intermediate COLLECT_PROCESSES
                        let inter_var = format!("_tmp_{}", target_var);
                        self.lower_assignment_value(&Expr::Call(chained.clone()), &inter_var, instructions, sequence)?;
                        inter_var
                    }
                    _ => return Err(IrGenError::UnsupportedExpression {
                        reason: "Filter source must be an identifier or collection call".to_string(),
                    }),
                };

                let predicate = self.lower_predicate(&filter.condition)?;
                *sequence += 1;
                let mut params = Map::new();
                params.insert("source_variable".to_string(), Value::String(source_var));
                params.insert("output_variable".to_string(), Value::String(target_var.to_string()));
                params.insert("predicate".to_string(), serde_json::to_value(predicate).map_err(|e| {
                    IrGenError::PredicateError { reason: e.to_string() }
                })?);

                instructions.push(IrInstruction {
                    sequence: *sequence,
                    op: IrOp::FilterProcesses.as_str().to_string(),
                    params,
                });
                Ok(())
            }

            _ => Err(IrGenError::UnsupportedExpression {
                reason: format!("Unsupported expression assigned to '{}'", target_var),
            }),
        }
    }

    fn lower_standalone_expression(
        &self,
        expr: &Expr,
        instructions: &mut Vec<IrInstruction>,
        sequence: &mut usize,
    ) -> Result<(), IrGenError> {
        if let Expr::Call(call) = expr {
            if let Expr::Identifier(ref mod_id) = *call.object {
                if mod_id.name == "Evidence" {
                    match call.function.name.as_str() {
                        "preserve" => {
                            *sequence += 1;
                            let source_var = self.extract_identifier_arg(&call.arguments, 0)?;
                            let tag = format!("{}_evidence", source_var);
                            let mut params = Map::new();
                            params.insert("source_variable".to_string(), Value::String(source_var));
                            params.insert("evidence_tag".to_string(), Value::String(tag));
                            instructions.push(IrInstruction {
                                sequence: *sequence,
                                op: IrOp::PreserveEvidence.as_str().to_string(),
                                params,
                            });
                            return Ok(());
                        }
                        "verify" => {
                            *sequence += 1;
                            let evidence_var = self.extract_identifier_arg(&call.arguments, 0)?;
                            let mut params = Map::new();
                            params.insert("evidence_variable".to_string(), Value::String(evidence_var));
                            instructions.push(IrInstruction {
                                sequence: *sequence,
                                op: IrOp::VerifyEvidence.as_str().to_string(),
                                params,
                            });
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }

        Err(IrGenError::UnsupportedExpression {
            reason: "Only Evidence.preserve and Evidence.verify can appear as standalone statements".to_string(),
        })
    }

    fn extract_identifier_arg(&self, args: &[Expr], index: usize) -> Result<String, IrGenError> {
        if index >= args.len() {
            return Err(IrGenError::UnsupportedExpression {
                reason: format!("Missing argument at index {}", index),
            });
        }
        match &args[index] {
            Expr::Identifier(ident) => Ok(ident.name.clone()),
            _ => Err(IrGenError::UnsupportedExpression {
                reason: format!("Expected identifier argument at index {}", index),
            }),
        }
    }

    /// Lower an AST expression inside a filter into the canonical Predicate tree.
    pub fn lower_predicate(&self, expr: &Expr) -> Result<Predicate, IrGenError> {
        match expr {
            Expr::BinaryLogical(binary) => {
                let left = Box::new(self.lower_predicate(&binary.left)?);
                let right = Box::new(self.lower_predicate(&binary.right)?);
                match binary.operator {
                    LogicalBinaryOp::And => Ok(Predicate::LogicalAnd { left, right }),
                    LogicalBinaryOp::Or  => Ok(Predicate::LogicalOr { left, right }),
                }
            }
            Expr::UnaryLogical(unary) => {
                let operand = Box::new(self.lower_predicate(&unary.expr)?);
                match unary.operator {
                    LogicalUnaryOp::Not => Ok(Predicate::LogicalNot { operand }),
                }
            }
            Expr::Comparison(comp) => {
                let field = match &*comp.left {
                    Expr::Identifier(ident) => ident.name.clone(),
                    _ => return Err(IrGenError::PredicateError {
                        reason: "Left side of comparison must be a process field identifier".to_string(),
                    }),
                };

                let op_str = match comp.operator {
                    ComparisonOp::EqualEqual   => "==",
                    ComparisonOp::BangEqual    => "!=",
                    ComparisonOp::Greater      => ">",
                    ComparisonOp::Less         => "<",
                    ComparisonOp::GreaterEqual => ">=",
                    ComparisonOp::LessEqual    => "<=",
                }.to_string();

                let literal = match &*comp.right {
                    Expr::Literal(lit) => match lit {
                        Literal::String(s) => PredicateLiteral::String { value: s.value.clone() },
                        Literal::Integer { value, .. } => PredicateLiteral::Integer { value: *value },
                        Literal::Float { value, .. } => PredicateLiteral::Float { value: *value },
                        Literal::Boolean { value, .. } => PredicateLiteral::Boolean { value: *value },
                        Literal::ByteSize { bytes, raw, .. } => PredicateLiteral::ByteSize {
                            value_bytes: *bytes,
                            raw_unit: raw.clone(),
                        },
                    },
                    _ => return Err(IrGenError::PredicateError {
                        reason: "Right side of comparison must be a literal".to_string(),
                    }),
                };

                Ok(Predicate::Comparison {
                    op: op_str,
                    field,
                    literal,
                })
            }
            _ => Err(IrGenError::PredicateError {
                reason: "Filter condition must evaluate to comparison or logical expression".to_string(),
            }),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// IR Generation Unit Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    fn dummy_span() -> Span {
        Span::new(1, 1)
    }

    fn ident(name: &str) -> Identifier {
        Identifier {
            name: name.to_string(),
            span: dummy_span(),
        }
    }

    fn str_lit(value: &str) -> StringLiteral {
        StringLiteral {
            value: value.to_string(),
            span: dummy_span(),
        }
    }

    fn module_call(module: &str, func: &str, args: Vec<Expr>) -> Expr {
        Expr::Call(Call {
            object: Box::new(Expr::Identifier(ident(module))),
            function: ident(func),
            arguments: args,
            span: dummy_span(),
        })
    }

    fn assign(target: &str, value: Expr) -> Statement {
        Statement::Assignment(Assignment {
            target: ident(target),
            type_annotation: None,
            value,
            span: dummy_span(),
        })
    }

    #[test]
    fn test_endpoint_triage_ir_generation_and_all_9_ops() {
        // Construct the AST for Endpoint Triage
        let inv = Investigation {
            name: str_lit("Endpoint Triage"),
            body: vec![
                assign("sys", module_call("System", "info", vec![])),
                assign("processes", module_call("Process", "collect", vec![])),
                assign("suspicious", Expr::Filter(Filter {
                    source: Box::new(Expr::Identifier(ident("processes"))),
                    condition: Box::new(Expr::BinaryLogical(BinaryLogical {
                        left: Box::new(Expr::Comparison(Comparison {
                            left: Box::new(Expr::Identifier(ident("name"))),
                            operator: ComparisonOp::EqualEqual,
                            right: Box::new(Expr::Literal(Literal::String(str_lit("powershell.exe")))),
                            span: dummy_span(),
                        })),
                        operator: LogicalBinaryOp::And,
                        right: Box::new(Expr::Comparison(Comparison {
                            left: Box::new(Expr::Identifier(ident("memory"))),
                            operator: ComparisonOp::Greater,
                            right: Box::new(Expr::Literal(Literal::ByteSize {
                                bytes: 524_288_000,
                                raw: "500MB".to_string(),
                                span: dummy_span(),
                            })),
                            span: dummy_span(),
                        })),
                        span: dummy_span(),
                    })),
                    span: dummy_span(),
                })),
                assign("netconns", module_call("Network", "connections", vec![])),
                assign("listeners", module_call("Network", "listeners", vec![])),
                assign("ev", module_call("Evidence", "preserve", vec![Expr::Identifier(ident("suspicious"))])),
                Statement::ExpressionStatement(module_call("Evidence", "verify", vec![Expr::Identifier(ident("ev"))])),
            ],
            span: dummy_span(),
        };

        let generator = IrGenerator::new("inv-2026-sih-001");
        let ir_doc = generator.generate_from_investigation(&inv).expect("IR generation should succeed");

        assert_eq!(ir_doc.ir_version, "v0.1.0");
        assert_eq!(ir_doc.investigation_id, "inv-2026-sih-001");
        assert_eq!(ir_doc.investigation_name, "Endpoint Triage");
        assert_eq!(ir_doc.target_scope, "ALL");

        // Verify exact 9 instructions in sequence
        let ops: Vec<&str> = ir_doc.instructions.iter().map(|i| i.op.as_str()).collect();
        assert_eq!(
            ops,
            vec![
                "INVESTIGATION_BEGIN",
                "COLLECT_SYSTEM_INFO",
                "COLLECT_PROCESSES",
                "FILTER_PROCESSES",
                "COLLECT_NETWORK_CONNECTIONS",
                "COLLECT_NETWORK_LISTENERS",
                "PRESERVE_EVIDENCE",
                "VERIFY_EVIDENCE",
                "INVESTIGATION_END",
            ]
        );

        // Verify sequences 1..=9
        for (i, inst) in ir_doc.instructions.iter().enumerate() {
            assert_eq!(inst.sequence, i + 1);
        }

        // Verify FILTER_PROCESSES predicate contains normalized bytes
        let filter_inst = &ir_doc.instructions[3];
        assert_eq!(filter_inst.op, "FILTER_PROCESSES");
        assert_eq!(filter_inst.params["source_variable"], "processes");
        assert_eq!(filter_inst.params["output_variable"], "suspicious");
        let predicate_json = serde_json::to_string(&filter_inst.params["predicate"]).unwrap();
        assert!(predicate_json.contains("524288000"));
        assert!(predicate_json.contains("powershell.exe"));

        // Verify serialization to JSON
        let json_str = serde_json::to_string_pretty(&ir_doc).unwrap();
        assert!(json_str.contains("\"ir_version\": \"v0.1.0\""));
    }
}
