//! JOCKY Forensic IR Validator
//!
//! Strict verification of Forensic IR documents against schema v0.1.0,
//! enforcing:
//! 1. JSON & schema structural validity
//! 2. Exact IR version ("v0.1.0")
//! 3. 9-instruction closed operation allowlist
//! 4. Required document and instruction fields
//! 5. Parameter presence, types, and validity
//! 6. Sequential instruction integrity (BEGIN -> ... -> END)
//!
//! Rejects any malformed, out-of-order, or unlisted operation (e.g. RUN_COMMAND).

use serde_json::Value;
use thiserror::Error;

use crate::ir::{ForensicIrDocument, IrInstruction, ALLOWED_OPERATIONS, IR_VERSION};

/// Diagnostic errors emitted during Forensic IR validation.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IrValidationError {
    #[error("Unsupported or unknown IR operation '{op}' in instruction {sequence}. Closed MVP allowlist: {allowlist}")]
    UnsupportedOperation {
        sequence: usize,
        op: String,
        allowlist: String,
    },

    #[error("Unsupported IR version '{version}'. Expected '{expected}'")]
    UnsupportedVersion {
        version: String,
        expected: String,
    },

    #[error("Missing required document field: '{field}'")]
    MissingDocumentField {
        field: String,
    },

    #[error("Missing required parameter '{param}' in instruction {sequence} ('{op}')")]
    MissingParameter {
        sequence: usize,
        op: String,
        param: String,
    },

    #[error("Invalid parameter type for '{param}' in instruction {sequence} ('{op}'): expected {expected}, found {actual}")]
    InvalidParameterType {
        sequence: usize,
        op: String,
        param: String,
        expected: String,
        actual: String,
    },

    #[error("Invalid parameter value for '{param}' in instruction {sequence} ('{op}'): {reason}")]
    InvalidParameterValue {
        sequence: usize,
        op: String,
        param: String,
        reason: String,
    },

    #[error("Malformed IR structure: {message}")]
    MalformedStructure {
        message: String,
    },

    #[error("JSON deserialization error: {0}")]
    JsonParseError(String),
}

/// The authoritative list of process fields valid in predicate filters
const ALLOWED_PROCESS_FIELDS: &[&str] = &["name", "pid", "ppid", "path", "cmdline", "memory"];

/// The authoritative comparison operators
const ALLOWED_COMPARISON_OPS: &[&str] = &["==", "!=", ">", "<", ">=", "<="];

/// IR Validator
pub struct IrValidator;

impl IrValidator {
    /// Parse and validate a raw JSON string into a validated ForensicIrDocument.
    pub fn validate_json(raw_json: &str) -> Result<ForensicIrDocument, IrValidationError> {
        let doc: ForensicIrDocument = serde_json::from_str(raw_json)
            .map_err(|e| IrValidationError::JsonParseError(e.to_string()))?;
        Self::validate(&doc)?;
        Ok(doc)
    }

    /// Validate an already parsed ForensicIrDocument.
    pub fn validate(doc: &ForensicIrDocument) -> Result<(), IrValidationError> {
        // 1. Version validation
        if doc.ir_version != IR_VERSION {
            return Err(IrValidationError::UnsupportedVersion {
                version: doc.ir_version.clone(),
                expected: IR_VERSION.to_string(),
            });
        }

        // 2. Document header fields
        if doc.investigation_id.trim().is_empty() {
            return Err(IrValidationError::MissingDocumentField {
                field: "investigation_id".to_string(),
            });
        }

        if doc.investigation_name.trim().is_empty() {
            return Err(IrValidationError::MissingDocumentField {
                field: "investigation_name".to_string(),
            });
        }

        if doc.target_scope.trim().is_empty() {
            return Err(IrValidationError::MissingDocumentField {
                field: "target_scope".to_string(),
            });
        }

        // 3. Instruction sequence existence
        if doc.instructions.is_empty() {
            return Err(IrValidationError::MalformedStructure {
                message: "Forensic IR contains zero instructions".to_string(),
            });
        }

        // 4. Boundary checks: Must start with INVESTIGATION_BEGIN and end with INVESTIGATION_END
        let first = &doc.instructions[0];
        if first.op != "INVESTIGATION_BEGIN" {
            return Err(IrValidationError::MalformedStructure {
                message: format!(
                    "First instruction must be 'INVESTIGATION_BEGIN', found '{}'",
                    first.op
                ),
            });
        }

        let last = &doc.instructions[doc.instructions.len() - 1];
        if last.op != "INVESTIGATION_END" {
            return Err(IrValidationError::MalformedStructure {
                message: format!(
                    "Last instruction must be 'INVESTIGATION_END', found '{}'",
                    last.op
                ),
            });
        }

        // 5. Instruction-by-instruction verification
        println!("Validator: starting instruction validation...");
        for (idx, inst) in doc.instructions.iter().enumerate() {
            let expected_seq = idx + 1;
            println!("Validator: instruction {} = {}", idx + 1, inst.op);
            if inst.sequence != expected_seq {
                return Err(IrValidationError::MalformedStructure {
                    message: format!(
                        "Instruction sequence mismatch at index {}: expected {}, found {}",
                        idx, expected_seq, inst.sequence
                    ),
                });
            }

            Self::validate_instruction(inst)?;
        }

        Ok(())
    }

    /// Validate a single IR instruction against the allowlist and parameter schema.
    pub fn validate_instruction(inst: &IrInstruction) -> Result<(), IrValidationError> {
        let op = &inst.op;
        let seq = inst.sequence;

        // A. Operation Allowlist Check
        if !ALLOWED_OPERATIONS.contains(&op.as_str()) {
            return Err(IrValidationError::UnsupportedOperation {
                sequence: seq,
                op: op.clone(),
                allowlist: ALLOWED_OPERATIONS.join(", "),
            });
        }

        // B. Parameter checks per instruction kind
        match op.as_str() {
            "INVESTIGATION_BEGIN" => {
                Self::require_str_param(inst, "investigation_id")?;
                Self::require_str_param(inst, "investigation_name")?;
                Self::require_str_param(inst, "timestamp_utc")?;
            }

            "COLLECT_SYSTEM_INFO" => {
                Self::require_str_param(inst, "output_variable")?;
            }

            "COLLECT_PROCESSES" => {
                Self::require_str_param(inst, "output_variable")?;
            }

            "FILTER_PROCESSES" => {
                Self::require_str_param(inst, "source_variable")?;
                Self::require_str_param(inst, "output_variable")?;
                let pred_val = Self::require_param(inst, "predicate")?;
                if !pred_val.is_object() {
                    return Err(IrValidationError::InvalidParameterType {
                        sequence: seq,
                        op: op.clone(),
                        param: "predicate".to_string(),
                        expected: "Object (Predicate AST)".to_string(),
                        actual: Self::json_type_name(pred_val).to_string(),
                    });
                }
                Self::validate_predicate_value(seq, op, pred_val)?;
            }

            "COLLECT_NETWORK_CONNECTIONS" => {
                Self::require_str_param(inst, "output_variable")?;
            }

            "COLLECT_NETWORK_LISTENERS" => {
                Self::require_str_param(inst, "output_variable")?;
            }

            "PRESERVE_EVIDENCE" => {
                Self::require_str_param(inst, "source_variable")?;
                Self::require_str_param(inst, "evidence_tag")?;
            }

            "VERIFY_EVIDENCE" => {
                Self::require_str_param(inst, "evidence_variable")?;
            }

            "INVESTIGATION_END" => {
                Self::require_str_param(inst, "investigation_id")?;
                Self::require_str_param(inst, "status")?;
            }

            _ => unreachable!(),
        }

        Ok(())
    }

    fn require_param<'a>(
        inst: &'a IrInstruction,
        name: &str,
    ) -> Result<&'a Value, IrValidationError> {
        inst.params.get(name).ok_or_else(|| IrValidationError::MissingParameter {
            sequence: inst.sequence,
            op: inst.op.clone(),
            param: name.to_string(),
        })
    }

    fn require_str_param(
        inst: &IrInstruction,
        name: &str,
    ) -> Result<String, IrValidationError> {
        let val = Self::require_param(inst, name)?;
        match val {
            Value::String(s) => {
                if s.trim().is_empty() {
                    Err(IrValidationError::InvalidParameterValue {
                        sequence: inst.sequence,
                        op: inst.op.clone(),
                        param: name.to_string(),
                        reason: "String parameter cannot be empty".to_string(),
                    })
                } else {
                    Ok(s.clone())
                }
            }
            other => Err(IrValidationError::InvalidParameterType {
                sequence: inst.sequence,
                op: inst.op.clone(),
                param: name.to_string(),
                expected: "String".to_string(),
                actual: Self::json_type_name(other).to_string(),
            }),
        }
    }

    fn validate_predicate_value(
        sequence: usize,
        op: &str,
        pred: &Value,
    ) -> Result<(), IrValidationError> {
        let obj = pred.as_object().ok_or_else(|| IrValidationError::InvalidParameterType {
            sequence,
            op: op.to_string(),
            param: "predicate".to_string(),
            expected: "Object".to_string(),
            actual: Self::json_type_name(pred).to_string(),
        })?;

        let pred_type = obj.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
            IrValidationError::InvalidParameterValue {
                sequence,
                op: op.to_string(),
                param: "predicate.type".to_string(),
                reason: "Predicate object must have a 'type' string".to_string(),
            }
        })?;

        match pred_type {
            "LogicalAnd" | "LogicalOr" => {
                let left = obj.get("left").ok_or_else(|| IrValidationError::InvalidParameterValue {
                    sequence,
                    op: op.to_string(),
                    param: "predicate.left".to_string(),
                    reason: format!("'{}' requires a 'left' sub-predicate", pred_type),
                })?;
                let right = obj.get("right").ok_or_else(|| IrValidationError::InvalidParameterValue {
                    sequence,
                    op: op.to_string(),
                    param: "predicate.right".to_string(),
                    reason: format!("'{}' requires a 'right' sub-predicate", pred_type),
                })?;
                Self::validate_predicate_value(sequence, op, left)?;
                Self::validate_predicate_value(sequence, op, right)?;
            }

            "LogicalNot" => {
                let operand = obj.get("operand").ok_or_else(|| IrValidationError::InvalidParameterValue {
                    sequence,
                    op: op.to_string(),
                    param: "predicate.operand".to_string(),
                    reason: "'LogicalNot' requires an 'operand' sub-predicate".to_string(),
                })?;
                Self::validate_predicate_value(sequence, op, operand)?;
            }

            "Comparison" => {
                let cmp_op = obj.get("op").and_then(|v| v.as_str()).ok_or_else(|| {
                    IrValidationError::InvalidParameterValue {
                        sequence,
                        op: op.to_string(),
                        param: "predicate.op".to_string(),
                        reason: "Comparison requires an 'op' string".to_string(),
                    }
                })?;

                if !ALLOWED_COMPARISON_OPS.contains(&cmp_op) {
                    return Err(IrValidationError::InvalidParameterValue {
                        sequence,
                        op: op.to_string(),
                        param: "predicate.op".to_string(),
                        reason: format!("Unsupported comparison operator '{}'. Expected one of: {:?}", cmp_op, ALLOWED_COMPARISON_OPS),
                    });
                }

                let field = obj.get("field").and_then(|v| v.as_str()).ok_or_else(|| {
                    IrValidationError::InvalidParameterValue {
                        sequence,
                        op: op.to_string(),
                        param: "predicate.field".to_string(),
                        reason: "Comparison requires a 'field' string".to_string(),
                    }
                })?;

                if !ALLOWED_PROCESS_FIELDS.contains(&field) {
                    return Err(IrValidationError::InvalidParameterValue {
                        sequence,
                        op: op.to_string(),
                        param: "predicate.field".to_string(),
                        reason: format!("Unknown process field '{}'. Expected one of: {:?}", field, ALLOWED_PROCESS_FIELDS),
                    });
                }

                let literal = obj.get("literal").and_then(|v| v.as_object()).ok_or_else(|| {
                    IrValidationError::InvalidParameterValue {
                        sequence,
                        op: op.to_string(),
                        param: "predicate.literal".to_string(),
                        reason: "Comparison requires a 'literal' object".to_string(),
                    }
                })?;

                let lit_type = literal.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
                    IrValidationError::InvalidParameterValue {
                        sequence,
                        op: op.to_string(),
                        param: "predicate.literal.type".to_string(),
                        reason: "Literal requires a 'type' string".to_string(),
                    }
                })?;

                match lit_type {
                    "String" | "Integer" | "Float" | "Boolean" => {
                        if !literal.contains_key("value") {
                            return Err(IrValidationError::InvalidParameterValue {
                                sequence,
                                op: op.to_string(),
                                param: "predicate.literal.value".to_string(),
                                reason: format!("Literal of type '{}' must contain 'value'", lit_type),
                            });
                        }
                    }
                    "ByteSize" => {
                        if !literal.contains_key("value_bytes") {
                            return Err(IrValidationError::InvalidParameterValue {
                                sequence,
                                op: op.to_string(),
                                param: "predicate.literal.value_bytes".to_string(),
                                reason: "ByteSize literal must contain 'value_bytes'".to_string(),
                            });
                        }
                    }
                    other => {
                        return Err(IrValidationError::InvalidParameterValue {
                            sequence,
                            op: op.to_string(),
                            param: "predicate.literal.type".to_string(),
                            reason: format!("Unknown literal type '{}'", other),
                        });
                    }
                }
            }

            other => {
                return Err(IrValidationError::InvalidParameterValue {
                    sequence,
                    op: op.to_string(),
                    param: "predicate.type".to_string(),
                    reason: format!("Unsupported predicate type '{}'", other),
                });
            }
        }

        Ok(())
    }

    fn json_type_name(val: &Value) -> &'static str {
        match val {
            Value::Null => "Null",
            Value::Bool(_) => "Boolean",
            Value::Number(_) => "Number",
            Value::String(_) => "String",
            Value::Array(_) => "Array",
            Value::Object(_) => "Object",
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// IR Validator Unit & Integration Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::ast::*;
    use crate::ir::IrGenerator;
    use crate::semantic::SemanticAnalyzer;

    fn dummy_span() -> Span {
        Span::new(1, 1)
    }

    fn sample_valid_endpoint_triage_ir_json() -> String {
        serde_json::to_string(&json!({
            "ir_version": "v0.1.0",
            "investigation_id": "inv-2026-sih-001",
            "investigation_name": "Endpoint Triage",
            "target_scope": "ALL",
            "instructions": [
                {
                    "sequence": 1,
                    "op": "INVESTIGATION_BEGIN",
                    "params": {
                        "investigation_id": "inv-2026-sih-001",
                        "investigation_name": "Endpoint Triage",
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
                    "params": { "output_variable": "processes" }
                },
                {
                    "sequence": 4,
                    "op": "FILTER_PROCESSES",
                    "params": {
                        "source_variable": "processes",
                        "output_variable": "suspicious",
                        "predicate": {
                            "type": "Comparison",
                            "op": "==",
                            "field": "name",
                            "literal": { "type": "String", "value": "powershell.exe" }
                        }
                    }
                },
                {
                    "sequence": 5,
                    "op": "COLLECT_NETWORK_CONNECTIONS",
                    "params": { "output_variable": "netconns" }
                },
                {
                    "sequence": 6,
                    "op": "COLLECT_NETWORK_LISTENERS",
                    "params": { "output_variable": "listeners" }
                },
                {
                    "sequence": 7,
                    "op": "PRESERVE_EVIDENCE",
                    "params": { "source_variable": "suspicious", "evidence_tag": "ev" }
                },
                {
                    "sequence": 8,
                    "op": "VERIFY_EVIDENCE",
                    "params": { "evidence_variable": "ev" }
                },
                {
                    "sequence": 9,
                    "op": "INVESTIGATION_END",
                    "params": { "investigation_id": "inv-2026-sih-001", "status": "SUCCESS" }
                }
            ]
        })).unwrap()
    }

    #[test]
    fn test_valid_ir_passes_validator() {
        let json_str = sample_valid_endpoint_triage_ir_json();
        let res = IrValidator::validate_json(&json_str);
        assert!(res.is_ok(), "Valid IR must pass validation: {:?}", res.err());
    }

    #[test]
    fn test_e2e_ast_to_semantic_to_ir_to_validator() {
        // Build AST for Endpoint Triage
        let inv = Investigation {
            name: StringLiteral {
                value: "Endpoint Triage".to_string(),
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
                Statement::Assignment(Assignment {
                    target: Identifier { name: "processes".to_string(), span: dummy_span() },
                    type_annotation: None,
                    value: Expr::Call(Call {
                        object: Box::new(Expr::Identifier(Identifier { name: "Process".to_string(), span: dummy_span() })),
                        function: Identifier { name: "collect".to_string(), span: dummy_span() },
                        arguments: vec![],
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                }),
                Statement::Assignment(Assignment {
                    target: Identifier { name: "suspicious".to_string(), span: dummy_span() },
                    type_annotation: None,
                    value: Expr::Filter(Filter {
                        source: Box::new(Expr::Identifier(Identifier { name: "processes".to_string(), span: dummy_span() })),
                        condition: Box::new(Expr::BinaryLogical(BinaryLogical {
                            left: Box::new(Expr::Comparison(Comparison {
                                left: Box::new(Expr::Identifier(Identifier { name: "name".to_string(), span: dummy_span() })),
                                operator: ComparisonOp::EqualEqual,
                                right: Box::new(Expr::Literal(Literal::String(StringLiteral {
                                    value: "powershell.exe".to_string(),
                                    span: dummy_span(),
                                }))),
                                span: dummy_span(),
                            })),
                            operator: LogicalBinaryOp::And,
                            right: Box::new(Expr::Comparison(Comparison {
                                left: Box::new(Expr::Identifier(Identifier { name: "memory".to_string(), span: dummy_span() })),
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
                    }),
                    span: dummy_span(),
                }),
                Statement::Assignment(Assignment {
                    target: Identifier { name: "netconns".to_string(), span: dummy_span() },
                    type_annotation: None,
                    value: Expr::Call(Call {
                        object: Box::new(Expr::Identifier(Identifier { name: "Network".to_string(), span: dummy_span() })),
                        function: Identifier { name: "connections".to_string(), span: dummy_span() },
                        arguments: vec![],
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                }),
                Statement::Assignment(Assignment {
                    target: Identifier { name: "listeners".to_string(), span: dummy_span() },
                    type_annotation: None,
                    value: Expr::Call(Call {
                        object: Box::new(Expr::Identifier(Identifier { name: "Network".to_string(), span: dummy_span() })),
                        function: Identifier { name: "listeners".to_string(), span: dummy_span() },
                        arguments: vec![],
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                }),
                Statement::Assignment(Assignment {
                    target: Identifier { name: "ev".to_string(), span: dummy_span() },
                    type_annotation: None,
                    value: Expr::Call(Call {
                        object: Box::new(Expr::Identifier(Identifier { name: "Evidence".to_string(), span: dummy_span() })),
                        function: Identifier { name: "preserve".to_string(), span: dummy_span() },
                        arguments: vec![Expr::Identifier(Identifier { name: "suspicious".to_string(), span: dummy_span() })],
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                }),
                Statement::ExpressionStatement(Expr::Call(Call {
                    object: Box::new(Expr::Identifier(Identifier { name: "Evidence".to_string(), span: dummy_span() })),
                    function: Identifier { name: "verify".to_string(), span: dummy_span() },
                    arguments: vec![Expr::Identifier(Identifier { name: "ev".to_string(), span: dummy_span() })],
                    span: dummy_span(),
                })),
            ],
            span: dummy_span(),
        };

        // Step 1: Semantic Analysis
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&inv).expect("Semantic analysis must pass");

        // Step 2: IR Generation
        let generator = IrGenerator::new("inv-2026-e2e");
        let ir_doc = generator.generate_from_investigation(&inv).expect("IR generation must succeed");

        // Step 3: IR Validation
        let val_res = IrValidator::validate(&ir_doc);
        assert!(val_res.is_ok(), "Generated IR must pass IR validator: {:?}", val_res.err());
    }

    #[test]
    fn test_reject_unknown_operation() {
        let mut val: Value = serde_json::from_str(&sample_valid_endpoint_triage_ir_json()).unwrap();
        // Tamper instruction 2 to have an arbitrary opcode
        val["instructions"][1]["op"] = json!("RUN_COMMAND");

        let res = IrValidator::validate_json(&val.to_string());
        assert!(res.is_err());
        match res.unwrap_err() {
            IrValidationError::UnsupportedOperation { op, .. } => {
                assert_eq!(op, "RUN_COMMAND");
            }
            other => panic!("Expected UnsupportedOperation, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_wrong_ir_version() {
        let mut val: Value = serde_json::from_str(&sample_valid_endpoint_triage_ir_json()).unwrap();
        val["ir_version"] = json!("v0.9.9");

        let res = IrValidator::validate_json(&val.to_string());
        assert!(res.is_err());
        match res.unwrap_err() {
            IrValidationError::UnsupportedVersion { version, expected } => {
                assert_eq!(version, "v0.9.9");
                assert_eq!(expected, "v0.1.0");
            }
            other => panic!("Expected UnsupportedVersion, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_missing_required_document_field() {
        let mut val: Value = serde_json::from_str(&sample_valid_endpoint_triage_ir_json()).unwrap();
        val["investigation_id"] = json!("");

        let res = IrValidator::validate_json(&val.to_string());
        assert!(res.is_err());
        match res.unwrap_err() {
            IrValidationError::MissingDocumentField { field } => {
                assert_eq!(field, "investigation_id");
            }
            other => panic!("Expected MissingDocumentField, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_missing_required_parameter() {
        let mut val: Value = serde_json::from_str(&sample_valid_endpoint_triage_ir_json()).unwrap();
        // Remove output_variable from COLLECT_SYSTEM_INFO
        val["instructions"][1]["params"] = json!({});

        let res = IrValidator::validate_json(&val.to_string());
        assert!(res.is_err());
        match res.unwrap_err() {
            IrValidationError::MissingParameter { op, param, .. } => {
                assert_eq!(op, "COLLECT_SYSTEM_INFO");
                assert_eq!(param, "output_variable");
            }
            other => panic!("Expected MissingParameter, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_wrong_parameter_type() {
        let mut val: Value = serde_json::from_str(&sample_valid_endpoint_triage_ir_json()).unwrap();
        // Give output_variable an integer instead of a string
        val["instructions"][1]["params"]["output_variable"] = json!(12345);

        let res = IrValidator::validate_json(&val.to_string());
        assert!(res.is_err());
        match res.unwrap_err() {
            IrValidationError::InvalidParameterType { param, expected, actual, .. } => {
                assert_eq!(param, "output_variable");
                assert_eq!(expected, "String");
                assert_eq!(actual, "Number");
            }
            other => panic!("Expected InvalidParameterType, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_invalid_parameter_value() {
        let mut val: Value = serde_json::from_str(&sample_valid_endpoint_triage_ir_json()).unwrap();
        // Set invalid comparison operator in predicate
        val["instructions"][3]["params"]["predicate"]["op"] = json!("LIKE");

        let res = IrValidator::validate_json(&val.to_string());
        assert!(res.is_err());
        match res.unwrap_err() {
            IrValidationError::InvalidParameterValue { param, reason, .. } => {
                assert_eq!(param, "predicate.op");
                assert!(reason.contains("Unsupported comparison operator"));
            }
            other => panic!("Expected InvalidParameterValue, got {:?}", other),
        }
    }

    #[test]
    fn test_reject_malformed_ir_structure() {
        let mut val: Value = serde_json::from_str(&sample_valid_endpoint_triage_ir_json()).unwrap();
        // Remove INVESTIGATION_BEGIN
        let mut instructions = val["instructions"].as_array().unwrap().clone();
        instructions.remove(0);
        val["instructions"] = json!(instructions);

        let res = IrValidator::validate_json(&val.to_string());
        assert!(res.is_err());
        match res.unwrap_err() {
            IrValidationError::MalformedStructure { message } => {
                assert!(message.contains("INVESTIGATION_BEGIN"));
            }
            other => panic!("Expected MalformedStructure, got {:?}", other),
        }
    }
}
