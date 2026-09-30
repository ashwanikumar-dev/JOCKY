// JOCKY AST — Compiler Frontend (M2)
//
// Abstract Syntax Tree node types for the JOCKY forensic DSL.
//
// Authoritative Specifications:
//   - JOCKY Grammar Specification v0.1 (M1 — September 2026)
//   - JOCKY 14-Day MVP Execution Plan §5 (Contract #2: AST Schema)
//   - JOCKY Engineering Handbook §14.3
//
// Frozen Contract Node Types:
//   Investigation, Assignment, Call, Filter, Literal, Identifier.
//
// The AST represents syntactic STRUCTURE, not semantic validity.
// Semantic validation (allowlists, type checking, field resolution) belongs to M3.

use jocky_lexer::Span;
use serde::{Deserialize, Serialize};

/// Serde remote definition for `jocky_lexer::Span` to enable serialization
/// without modifying the lexer crate.
#[derive(Serialize, Deserialize)]
#[serde(remote = "Span")]
pub struct SpanDef {
    pub line: usize,
    pub column: usize,
}

/// A complete JOCKY program representing exactly one investigation block.
///
/// Grammar v0.1 §3.1, §3.2:
///   Program := InvestigationBlock ;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub investigation: Investigation,
}

/// A statement inside an investigation block body.
///
/// Grammar v0.1 §3.2:
///   Statement := Assignment | ExpressionStatement ;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    /// A variable assignment: `target = expr` or `target: TypeName = expr`.
    Assignment(Assignment),
    /// A standalone expression statement, e.g. `Evidence.preserve(processes)`.
    ExpressionStatement(Expr),
}

// ── Frozen AST Node Types ────────────────────────────────────────────

/// An investigation block: `investigation "name" { statements... }`.
///
/// Grammar v0.1 §3.2, §10:
///   InvestigationBlock := "investigation" StringLiteral "{" { Statement } "}" ;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Investigation {
    /// The investigation name title.
    pub name: StringLiteral,
    /// Ordered list of statements inside the investigation block.
    pub body: Vec<Statement>,
    /// Source position of the `investigation` keyword.
    #[serde(with = "SpanDef")]
    pub span: Span,
}

/// A string literal value, e.g. `"Host Check"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StringLiteral {
    /// The string value.
    pub value: String,
    #[serde(with = "SpanDef")]
    pub span: Span,
}

/// A variable assignment: `target = expr` or `target: TypeName = expr`.
///
/// Grammar v0.1 §3.2, §10:
///   Assignment := Identifier [ ":" TypeName ] "=" Expression ;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    /// The variable name being assigned to.
    pub target: Identifier,
    /// Optional type annotation (e.g. `ProcessSet`).
    pub type_annotation: Option<Identifier>,
    /// The right-hand expression subtree.
    pub value: Expr,
    #[serde(with = "SpanDef")]
    pub span: Span,
}

/// An expression in the JOCKY DSL.
///
/// Grammar v0.1 §3.2:
///   Expression := LogicalOrExpr ;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// A module.function() or method call, e.g. `System.info()`.
    Call(Call),
    /// A `.filter(condition)` application on an expression.
    Filter(Filter),
    /// A binary comparison expression, e.g. `memory > 500MB`.
    Comparison(Comparison),
    /// A binary logical expression, e.g. `a and b`, `a or b`.
    BinaryLogical(BinaryLogical),
    /// A unary logical expression, e.g. `not a`.
    UnaryLogical(UnaryLogical),
    /// An identifier reference (variable, field, or module name).
    Identifier(Identifier),
    /// A literal value (string, integer, float, boolean, byte-size).
    Literal(Literal),
}

/// A function or method call: `Module.function(args...)` or `expr.method(args...)`.
///
/// Grammar v0.1 §3.2, §10:
///   Postfix step: Receiver subtree, called method name, argument subtrees.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Call {
    /// The receiver / object the function is called on.
    pub object: Box<Expr>,
    /// The called function/method name.
    pub function: Identifier,
    /// Ordered list of argument expression subtrees.
    pub arguments: Vec<Expr>,
    #[serde(with = "SpanDef")]
    pub span: Span,
}

/// A filter expression: `source.filter(condition)`.
///
/// Grammar v0.1 §3.2, §6, §10:
///   The condition is a full expression subtree that can evaluate to
///   comparisons, logical combinators (`and`, `or`, `not`), and grouping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Filter {
    /// The expression being filtered (e.g. `Process.collect()` or `processes`).
    pub source: Box<Expr>,
    /// The filter condition expression subtree.
    pub condition: Box<Expr>,
    #[serde(with = "SpanDef")]
    pub span: Span,
}

/// A binary comparison expression: `left operator right`.
///
/// Grammar v0.1 §3.2, §6.3, §10:
///   ComparisonExpr := Postfix [ ComparisonOp Postfix ] ;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comparison {
    /// Left operand subtree.
    pub left: Box<Expr>,
    /// Comparison operator enum.
    pub operator: ComparisonOp,
    /// Right operand subtree.
    pub right: Box<Expr>,
    #[serde(with = "SpanDef")]
    pub span: Span,
}

/// Comparison operators supported by JOCKY.
///
/// Grammar v0.1 §2.4, §6.3:
///   `==`, `!=`, `>`, `<`, `>=`, `<=`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOp {
    /// `==` — Equal to
    EqualEqual,
    /// `!=` — Not equal to
    BangEqual,
    /// `>` — Greater than
    Greater,
    /// `<` — Less than
    Less,
    /// `>=` — Greater than or equal to
    GreaterEqual,
    /// `<=` — Less than or equal to
    LessEqual,
}

/// A binary logical expression: `left operator right`.
///
/// Grammar v0.1 §3.2, §3.4, §6.6, §10:
///   LogicalOrExpr  := LogicalAndExpr { "or" LogicalAndExpr } ;
///   LogicalAndExpr := NotExpr { "and" NotExpr } ;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryLogical {
    /// Left operand subtree.
    pub left: Box<Expr>,
    /// Logical binary operator (`and`, `or`).
    pub operator: LogicalBinaryOp,
    /// Right operand subtree.
    pub right: Box<Expr>,
    #[serde(with = "SpanDef")]
    pub span: Span,
}

/// Binary logical operators.
///
/// Grammar v0.1 §2.2, §3.2:
///   `and`, `or`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogicalBinaryOp {
    /// `and` — logical conjunction
    And,
    /// `or` — logical disjunction
    Or,
}

/// A unary logical expression: `operator expr`.
///
/// Grammar v0.1 §3.2, §3.4, §6.8, §10:
///   NotExpr := [ "not" ] ComparisonExpr ;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnaryLogical {
    /// Unary logical operator (`not`).
    pub operator: LogicalUnaryOp,
    /// Target expression subtree.
    pub expr: Box<Expr>,
    #[serde(with = "SpanDef")]
    pub span: Span,
}

/// Unary logical operators.
///
/// Grammar v0.1 §2.2, §3.2:
///   `not`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogicalUnaryOp {
    /// `not` — logical negation
    Not,
}

/// An identifier: variable name, module name, function name, field name, type name.
///
/// Grammar v0.1 §2.3, §10
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Identifier {
    /// The identifier name text.
    pub name: String,
    #[serde(with = "SpanDef")]
    pub span: Span,
}

/// A literal value.
///
/// Grammar v0.1 §2.8, §10:
///   String, Integer, Float, Boolean, ByteSize.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    /// A string literal, e.g. `"Host Check"`.
    String(StringLiteral),
    /// An integer literal, e.g. `500`.
    Integer {
        value: i64,
        #[serde(with = "SpanDef")]
        span: Span,
    },
    /// A floating-point literal, e.g. `3.14`.
    Float {
        value: f64,
        #[serde(with = "SpanDef")]
        span: Span,
    },
    /// A boolean literal: `true` or `false`.
    Boolean {
        value: bool,
        #[serde(with = "SpanDef")]
        span: Span,
    },
    /// A byte-size literal, e.g. `500MB`, normalized to byte count.
    /// Grammar v0.1 §2.8.5, §10
    ByteSize {
        /// Normalized size in bytes (e.g. 500 * 1024 * 1024).
        bytes: u64,
        /// Raw original literal string (e.g. "500MB").
        raw: String,
        #[serde(with = "SpanDef")]
        span: Span,
    },
}

// ── Unit Tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_span() -> Span {
        Span { line: 1, column: 1 }
    }

    #[test]
    fn test_ast_construction_and_json_roundtrip() {
        // Build a complete Program matching Grammar v0.1 realistic example:
        // investigation "Endpoint Triage" {
        //     processes = Process.collect()
        //     suspicious = processes.filter(name == "powershell.exe" and memory > 500MB)
        //     Evidence.preserve(suspicious)
        // }
        let program = Program {
            investigation: Investigation {
                name: StringLiteral {
                    value: "Endpoint Triage".to_string(),
                    span: dummy_span(),
                },
                body: vec![
                    // processes = Process.collect()
                    Statement::Assignment(Assignment {
                        target: Identifier {
                            name: "processes".to_string(),
                            span: dummy_span(),
                        },
                        type_annotation: None,
                        value: Expr::Call(Call {
                            object: Box::new(Expr::Identifier(Identifier {
                                name: "Process".to_string(),
                                span: dummy_span(),
                            })),
                            function: Identifier {
                                name: "collect".to_string(),
                                span: dummy_span(),
                            },
                            arguments: vec![],
                            span: dummy_span(),
                        }),
                        span: dummy_span(),
                    }),
                    // suspicious = processes.filter(name == "powershell.exe" and memory > 500MB)
                    Statement::Assignment(Assignment {
                        target: Identifier {
                            name: "suspicious".to_string(),
                            span: dummy_span(),
                        },
                        type_annotation: None,
                        value: Expr::Filter(Filter {
                            source: Box::new(Expr::Identifier(Identifier {
                                name: "processes".to_string(),
                                span: dummy_span(),
                            })),
                            condition: Box::new(Expr::BinaryLogical(BinaryLogical {
                                left: Box::new(Expr::Comparison(Comparison {
                                    left: Box::new(Expr::Identifier(Identifier {
                                        name: "name".to_string(),
                                        span: dummy_span(),
                                    })),
                                    operator: ComparisonOp::EqualEqual,
                                    right: Box::new(Expr::Literal(Literal::String(
                                        StringLiteral {
                                            value: "powershell.exe".to_string(),
                                            span: dummy_span(),
                                        },
                                    ))),
                                    span: dummy_span(),
                                })),
                                operator: LogicalBinaryOp::And,
                                right: Box::new(Expr::Comparison(Comparison {
                                    left: Box::new(Expr::Identifier(Identifier {
                                        name: "memory".to_string(),
                                        span: dummy_span(),
                                    })),
                                    operator: ComparisonOp::Greater,
                                    right: Box::new(Expr::Literal(Literal::ByteSize {
                                        bytes: 500 * 1024 * 1024,
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
                    // Standalone ExpressionStatement: Evidence.preserve(suspicious)
                    Statement::ExpressionStatement(Expr::Call(Call {
                        object: Box::new(Expr::Identifier(Identifier {
                            name: "Evidence".to_string(),
                            span: dummy_span(),
                        })),
                        function: Identifier {
                            name: "preserve".to_string(),
                            span: dummy_span(),
                        },
                        arguments: vec![Expr::Identifier(Identifier {
                            name: "suspicious".to_string(),
                            span: dummy_span(),
                        })],
                        span: dummy_span(),
                    })),
                ],
                span: dummy_span(),
            },
        };

        // Serialize to JSON.
        let json = serde_json::to_string(&program).expect("AST must serialize to JSON");
        assert!(json.contains("Endpoint Triage"));
        assert!(json.contains("EqualEqual"));
        assert!(json.contains("500MB"));

        // Deserialize back from JSON and verify equality.
        let deserialized: Program =
            serde_json::from_str(&json).expect("AST must deserialize from JSON");
        assert_eq!(program, deserialized);
    }

    #[test]
    fn test_all_comparison_and_logical_operators() {
        let ops = vec![
            ComparisonOp::EqualEqual,
            ComparisonOp::BangEqual,
            ComparisonOp::Greater,
            ComparisonOp::Less,
            ComparisonOp::GreaterEqual,
            ComparisonOp::LessEqual,
        ];
        for op in ops {
            let json = serde_json::to_string(&op).unwrap();
            let de: ComparisonOp = serde_json::from_str(&json).unwrap();
            assert_eq!(op, de);
        }

        let and_op = LogicalBinaryOp::And;
        let or_op = LogicalBinaryOp::Or;
        let not_op = LogicalUnaryOp::Not;
        assert_eq!(
            and_op,
            serde_json::from_str(&serde_json::to_string(&and_op).unwrap()).unwrap()
        );
        assert_eq!(
            or_op,
            serde_json::from_str(&serde_json::to_string(&or_op).unwrap()).unwrap()
        );
        assert_eq!(
            not_op,
            serde_json::from_str(&serde_json::to_string(&not_op).unwrap()).unwrap()
        );
    }
}
