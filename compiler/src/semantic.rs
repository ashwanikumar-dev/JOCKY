use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use crate::ast::{
    Call, Expr, Filter, Investigation, Literal, Program, Span, Statement,
};

// ──────────────────────────────────────────────────────────────────────
// Type System v0.1
// ──────────────────────────────────────────────────────────────────────

/// The 10 types in the JOCKY Type System v0.1.
/// Every expression in a valid JOCKY script must resolve to one of these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JockyType {
    SystemInfo,
    ProcessSet,
    ConnectionSet,
    ListenerSet,
    RawEvidence,
    Boolean,
    String,
    Integer,
    Float,
    ByteSize,
    Void,
}

impl fmt::Display for JockyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JockyType::SystemInfo     => write!(f, "SystemInfo"),
            JockyType::ProcessSet     => write!(f, "ProcessSet"),
            JockyType::ConnectionSet  => write!(f, "ConnectionSet"),
            JockyType::ListenerSet    => write!(f, "ListenerSet"),
            JockyType::RawEvidence    => write!(f, "RawEvidence"),
            JockyType::Boolean        => write!(f, "Boolean"),
            JockyType::String         => write!(f, "String"),
            JockyType::Integer        => write!(f, "Integer"),
            JockyType::Float          => write!(f, "Float"),
            JockyType::ByteSize       => write!(f, "ByteSize"),
            JockyType::Void           => write!(f, "Void"),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// Semantic Errors — clear, actionable diagnostics per Section 10
// ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SemanticError {
    #[error("Unknown module '{name}' at line {line}, col {col}. Only 'System', 'Process', 'Network', 'Evidence' are permitted.")]
    UnknownModule {
        name: String,
        line: usize,
        col: usize,
    },

    #[error("Unknown function '{module}.{function}' at line {line}, col {col}. Not in the frozen 7-call allowlist.")]
    UnknownFunction {
        module: String,
        function: String,
        line: usize,
        col: usize,
    },

    #[error("Function '{module}.{function}' expects {expected} argument(s), found {actual} at line {line}, col {col}.")]
    ArgumentCountMismatch {
        module: String,
        function: String,
        expected: usize,
        actual: usize,
        line: usize,
        col: usize,
    },

    #[error("Type mismatch in '{module}.{function}' arg {arg_index} at line {line}, col {col}. Expected {expected}, got {actual}.")]
    TypeMismatch {
        module: String,
        function: String,
        arg_index: usize,
        expected: String,
        actual: String,
        line: usize,
        col: usize,
    },

    #[error("Undefined variable '{name}' used at line {line}, col {col}.")]
    UndefinedVariable {
        name: String,
        line: usize,
        col: usize,
    },

    #[error("Cannot filter on type '{actual}' at line {line}, col {col}. Filtering is only supported on 'ProcessSet'.")]
    InvalidFilterTarget {
        actual: String,
        line: usize,
        col: usize,
    },

    #[error("Empty investigation block at line {line}, col {col}. An investigation must contain at least one operation.")]
    EmptyInvestigation {
        line: usize,
        col: usize,
    },

    #[error("Type annotation mismatch for variable '{variable}' at line {line}, col {col}. Declared as '{declared}' but expression evaluates to '{inferred}'.")]
    AnnotationMismatch {
        variable: String,
        declared: String,
        inferred: String,
        line: usize,
        col: usize,
    },
}

// ──────────────────────────────────────────────────────────────────────
// Semantic Analyzer
// ──────────────────────────────────────────────────────────────────────

/// Implicit field names accessible inside a ProcessSet.filter() predicate.
/// These are the columns of a process snapshot in JOCKY v0.1.
const PROCESS_FIELDS: &[(&str, JockyType)] = &[
    ("name",    JockyType::String),
    ("pid",     JockyType::Integer),
    ("ppid",    JockyType::Integer),
    ("path",    JockyType::String),
    ("cmdline", JockyType::String),
    ("memory",  JockyType::ByteSize),
];

pub struct SemanticAnalyzer {
    symbol_table: HashMap<String, JockyType>,
    /// When true, implicit process-domain field names (name, pid, etc.) are
    /// valid identifiers inside the currently-analyzed expression.
    in_filter_context: bool,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            in_filter_context: false,
        }
    }

    /// Convenience entry point: analyzes a full Program AST root
    pub fn analyze_program(&mut self, program: &Program) -> Result<(), SemanticError> {
        self.analyze(&program.investigation)
    }

    /// Primary entry point: analyzes an Investigation AST node
    pub fn analyze(&mut self, inv: &Investigation) -> Result<(), SemanticError> {
        if inv.body.is_empty() {
            return Err(SemanticError::EmptyInvestigation {
                line: inv.span.line,
                col: inv.span.column,
            });
        }

        self.symbol_table.clear();

        for stmt in &inv.body {
            self.analyze_statement(stmt)?;
        }

        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<(), SemanticError> {
        match stmt {
            Statement::Assignment(assignment) => {
                let inferred_type = self.analyze_expression(&assignment.value)?;

                // If an explicit type annotation is present, check matching
                if let Some(annotated) = &assignment.type_annotation {
                    let expected_type = Self::parse_type_name(&annotated.name);
                    if let Some(ref expected) = expected_type {
                        if *expected != inferred_type {
                            return Err(SemanticError::AnnotationMismatch {
                                variable: assignment.target.name.clone(),
                                declared: annotated.name.clone(),
                                inferred: inferred_type.to_string(),
                                line: annotated.span.line,
                                col: annotated.span.column,
                            });
                        }
                    }
                    // If type name is unrecognized, we allow it through (forward compat)
                }

                self.symbol_table.insert(assignment.target.name.clone(), inferred_type);
                Ok(())
            }
            Statement::ExpressionStatement(expr) => {
                self.analyze_expression(expr)?;
                Ok(())
            }
        }
    }

    /// Map a user-written type annotation string to our internal JockyType
    fn parse_type_name(name: &str) -> Option<JockyType> {
        match name {
            "SystemInfo"    => Some(JockyType::SystemInfo),
            "ProcessSet"    => Some(JockyType::ProcessSet),
            "ConnectionSet" => Some(JockyType::ConnectionSet),
            "ListenerSet"   => Some(JockyType::ListenerSet),
            "RawEvidence"   => Some(JockyType::RawEvidence),
            "Boolean"       => Some(JockyType::Boolean),
            "String"        => Some(JockyType::String),
            "Integer"       => Some(JockyType::Integer),
            "Float"         => Some(JockyType::Float),
            "ByteSize"      => Some(JockyType::ByteSize),
            _               => None,
        }
    }

    fn analyze_expression(&mut self, expr: &Expr) -> Result<JockyType, SemanticError> {
        match expr {
            Expr::Literal(lit) => Ok(match lit {
                Literal::String(_) => JockyType::String,
                Literal::Integer { .. } => JockyType::Integer,
                Literal::Float { .. } => JockyType::Float,
                Literal::Boolean { .. } => JockyType::Boolean,
                Literal::ByteSize { .. } => JockyType::ByteSize,
            }),

            Expr::Identifier(ident) => {
                // First check the symbol table
                if let Some(ty) = self.symbol_table.get(&ident.name) {
                    return Ok(ty.clone());
                }
                // Inside a filter predicate, allow implicit process fields
                if self.in_filter_context {
                    for &(field_name, ref field_type) in PROCESS_FIELDS {
                        if ident.name == field_name {
                            return Ok(field_type.clone());
                        }
                    }
                }
                // Check if it's an uppercase module identifier used out of call context
                if ident.name.chars().next().map_or(false, |c| c.is_uppercase()) {
                    match ident.name.as_str() {
                        "System" | "Process" | "Network" | "Evidence" => Ok(JockyType::Void),
                        _ => Err(SemanticError::UnknownModule {
                            name: ident.name.clone(),
                            line: ident.span.line,
                            col: ident.span.column,
                        }),
                    }
                } else {
                    Err(SemanticError::UndefinedVariable {
                        name: ident.name.clone(),
                        line: ident.span.line,
                        col: ident.span.column,
                    })
                }
            }

            Expr::Call(call) => self.analyze_call(call),

            Expr::Filter(filter) => self.analyze_filter(filter),

            Expr::Comparison(comp) => {
                let _l_type = self.analyze_expression(&comp.left)?;
                let _r_type = self.analyze_expression(&comp.right)?;
                Ok(JockyType::Boolean)
            }

            Expr::BinaryLogical(binary) => {
                let _l_type = self.analyze_expression(&binary.left)?;
                let _r_type = self.analyze_expression(&binary.right)?;
                Ok(JockyType::Boolean)
            }

            Expr::UnaryLogical(unary) => {
                let _op_type = self.analyze_expression(&unary.expr)?;
                Ok(JockyType::Boolean)
            }
        }
    }

    fn analyze_call(&mut self, call: &Call) -> Result<JockyType, SemanticError> {
        let fn_name = &call.function.name;
        let span = call.span.clone();
        let args = &call.arguments;

        // Check if the object is a module identifier (e.g. System.info())
        if let Expr::Identifier(ref mod_id) = *call.object {
            match mod_id.name.as_str() {
                "System" => {
                    if fn_name != "info" {
                        return Err(SemanticError::UnknownFunction {
                            module: "System".to_string(),
                            function: fn_name.clone(),
                            line: span.line,
                            col: span.column,
                        });
                    }
                    self.assert_arg_count("System", fn_name, 0, args.len(), span)?;
                    return Ok(JockyType::SystemInfo);
                }

                "Process" => {
                    if fn_name != "collect" {
                        return Err(SemanticError::UnknownFunction {
                            module: "Process".to_string(),
                            function: fn_name.clone(),
                            line: span.line,
                            col: span.column,
                        });
                    }
                    self.assert_arg_count("Process", fn_name, 0, args.len(), span)?;
                    return Ok(JockyType::ProcessSet);
                }

                "Network" => match fn_name.as_str() {
                    "connections" => {
                        self.assert_arg_count("Network", fn_name, 0, args.len(), span)?;
                        return Ok(JockyType::ConnectionSet);
                    }
                    "listeners" => {
                        self.assert_arg_count("Network", fn_name, 0, args.len(), span)?;
                        return Ok(JockyType::ListenerSet);
                    }
                    _ => {
                        return Err(SemanticError::UnknownFunction {
                            module: "Network".to_string(),
                            function: fn_name.clone(),
                            line: span.line,
                            col: span.column,
                        });
                    }
                },

                "Evidence" => return match fn_name.as_str() {
                    "preserve" => {
                        self.assert_arg_count("Evidence", fn_name, 1, args.len(), span.clone())?;
                        let arg_type = self.analyze_expression(&args[0])?;
                        match arg_type {
                            JockyType::SystemInfo
                            | JockyType::ProcessSet
                            | JockyType::ConnectionSet
                            | JockyType::ListenerSet => Ok(JockyType::RawEvidence),
                            other => Err(SemanticError::TypeMismatch {
                                module: "Evidence".to_string(),
                                function: "preserve".to_string(),
                                arg_index: 0,
                                expected: "ForensicArtifact (SystemInfo, ProcessSet, ConnectionSet, ListenerSet)".to_string(),
                                actual: other.to_string(),
                                line: span.line,
                                col: span.column,
                            }),
                        }
                    }
                    "verify" => {
                        self.assert_arg_count("Evidence", fn_name, 1, args.len(), span.clone())?;
                        let arg_type = self.analyze_expression(&args[0])?;
                        if arg_type != JockyType::RawEvidence {
                            return Err(SemanticError::TypeMismatch {
                                module: "Evidence".to_string(),
                                function: "verify".to_string(),
                                arg_index: 0,
                                expected: "RawEvidence".to_string(),
                                actual: arg_type.to_string(),
                                line: span.line,
                                col: span.column,
                            });
                        }
                        Ok(JockyType::Boolean)
                    }
                    _ => Err(SemanticError::UnknownFunction {
                        module: "Evidence".to_string(),
                        function: fn_name.clone(),
                        line: span.line,
                        col: span.column,
                    }),
                },

                // If it starts with uppercase and is not a known module, reject as unknown module
                other_mod if other_mod.chars().next().map_or(false, |c| c.is_uppercase())
                    && !self.symbol_table.contains_key(other_mod) =>
                {
                    return Err(SemanticError::UnknownModule {
                        name: other_mod.to_string(),
                        line: mod_id.span.line,
                        col: mod_id.span.column,
                    });
                }

                _ => {
                    // Falls through to receiver object evaluation below
                }
            }
        }

        // Method call on a variable or chained expression
        let receiver_type = self.analyze_expression(&call.object)?;

        if fn_name == "filter" {
            if receiver_type != JockyType::ProcessSet {
                return Err(SemanticError::InvalidFilterTarget {
                    actual: receiver_type.to_string(),
                    line: span.line,
                    col: span.column,
                });
            }
            self.assert_arg_count("ProcessSet", "filter", 1, args.len(), span)?;
            let prev = self.in_filter_context;
            self.in_filter_context = true;
            let result = self.analyze_expression(&args[0]);
            self.in_filter_context = prev;
            let _ = result?;
            Ok(JockyType::ProcessSet)
        } else {
            Err(SemanticError::UnknownFunction {
                module: receiver_type.to_string(),
                function: fn_name.clone(),
                line: span.line,
                col: span.column,
            })
        }
    }

    fn analyze_filter(&mut self, filter: &Filter) -> Result<JockyType, SemanticError> {
        let source_type = self.analyze_expression(&filter.source)?;
        if source_type != JockyType::ProcessSet {
            return Err(SemanticError::InvalidFilterTarget {
                actual: source_type.to_string(),
                line: filter.span.line,
                col: filter.span.column,
            });
        }

        let prev = self.in_filter_context;
        self.in_filter_context = true;
        let result = self.analyze_expression(&filter.condition);
        self.in_filter_context = prev;
        let _ = result?;

        Ok(JockyType::ProcessSet)
    }

    fn assert_arg_count(
        &mut self,
        module: &str,
        function: &str,
        expected: usize,
        actual: usize,
        span: Span,
    ) -> Result<(), SemanticError> {
        if expected != actual {
            Err(SemanticError::ArgumentCountMismatch {
                module: module.to_string(),
                function: function.to_string(),
                expected,
                actual,
                line: span.line,
                col: span.column,
            })
        } else {
            Ok(())
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
//  Day 4 Test Suite — 5 Valid Scripts + 5 Invalid Scripts
// ══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    /// Helper: construct a Span for line/column
    fn span(line: usize, col: usize) -> Span {
        Span::new(line, col)
    }

    /// Helper: construct a simple identifier
    fn ident(name: &str, line: usize, col: usize) -> Identifier {
        Identifier {
            name: name.to_string(),
            span: span(line, col),
        }
    }

    /// Helper: construct a StringLiteral
    fn str_lit(val: &str, line: usize, col: usize) -> StringLiteral {
        StringLiteral {
            value: val.to_string(),
            span: span(line, col),
        }
    }

    /// Helper: construct a module method call like `Module.function(args...)`
    fn module_call(
        module: &str, function: &str,
        args: Vec<Expr>,
        line: usize, col: usize,
    ) -> Expr {
        Expr::Call(Call {
            object: Box::new(Expr::Identifier(ident(module, line, col))),
            function: ident(function, line, col + module.len() + 1),
            arguments: args,
            span: span(line, col + module.len() + 1),
        })
    }

    /// Helper: construct a variable reference expression
    fn var_expr(name: &str, line: usize, col: usize) -> Expr {
        Expr::Identifier(ident(name, line, col))
    }

    /// Helper: construct a method call like `variable.filter(condition)`
    fn filter_call(
        source_var: &str,
        condition: Expr,
        line: usize, col: usize,
    ) -> Expr {
        Expr::Filter(Filter {
            source: Box::new(Expr::Identifier(ident(source_var, line, col))),
            condition: Box::new(condition),
            span: span(line, col + source_var.len() + 1),
        })
    }

    /// Helper: construct an arbitrary method call on a variable
    #[allow(dead_code)]
    fn method_call(
        var_name: &str, method: &str,
        args: Vec<Expr>,
        line: usize, col: usize,
    ) -> Expr {
        Expr::Call(Call {
            object: Box::new(Expr::Identifier(ident(var_name, line, col))),
            function: ident(method, line, col + var_name.len() + 1),
            arguments: args,
            span: span(line, col + var_name.len() + 1),
        })
    }

    /// Helper: make an assignment statement
    fn assign(
        target: &str, annotation: Option<&str>,
        value: Expr,
        line: usize, col: usize,
    ) -> Statement {
        Statement::Assignment(Assignment {
            target: ident(target, line, col),
            type_annotation: annotation.map(|a| ident(a, line, col + target.len() + 2)),
            value,
            span: span(line, col),
        })
    }

    /// Helper: wrap an expression as a standalone statement
    #[allow(dead_code)]
    fn expr_stmt(expr: Expr) -> Statement {
        Statement::ExpressionStatement(expr)
    }

    /// Helper: create an investigation block
    fn investigation(title: &str, body: Vec<Statement>) -> Investigation {
        Investigation {
            name: str_lit(title, 1, 15),
            body,
            span: span(1, 1),
        }
    }

    // ----------------------------------------------------------------
    //  ✅ VALID SCRIPT 1: Full endpoint triage (all 7 API calls used)
    //
    //  investigation "Endpoint Triage" {
    //      sys = System.info()
    //      sys_ev = Evidence.preserve(sys)
    //      procs = Process.collect()
    //      suspicious = procs.filter(name == "powershell.exe")
    //      proc_ev = Evidence.preserve(suspicious)
    //      ok = Evidence.verify(proc_ev)
    //  }
    // ----------------------------------------------------------------
    #[test]
    fn valid_01_full_endpoint_triage() {
        let inv = investigation(
            "Endpoint Triage",
            vec![
                assign("sys", None, module_call("System", "info", vec![], 2, 5), 2, 5),
                assign("sys_ev", None, module_call("Evidence", "preserve", vec![var_expr("sys", 3, 35)], 3, 5), 3, 5),
                assign("procs", None, module_call("Process", "collect", vec![], 4, 5), 4, 5),
                // procs.filter(name == "powershell.exe")
                assign("suspicious", None, filter_call(
                    "procs",
                    Expr::Comparison(Comparison {
                        left: Box::new(Expr::Identifier(ident("name", 5, 30))),
                        operator: ComparisonOp::EqualEqual,
                        right: Box::new(Expr::Literal(Literal::String(str_lit("powershell.exe", 5, 38)))),
                        span: span(5, 30),
                    }),
                    5, 5,
                ), 5, 5),
                assign("proc_ev", None, module_call("Evidence", "preserve", vec![var_expr("suspicious", 6, 37)], 6, 5), 6, 5),
                assign("ok", None, module_call("Evidence", "verify", vec![var_expr("proc_ev", 7, 32)], 7, 5), 7, 5),
            ],
        );

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&inv);
        assert!(result.is_ok(), "Valid script 1 (full endpoint triage) should pass: {:?}", result.err());
    }

    // ----------------------------------------------------------------
    //  ✅ VALID SCRIPT 2: Network snapshot (connections + listeners)
    //
    //  investigation "Network Snapshot" {
    //      conns = Network.connections()
    //      listeners = Network.listeners()
    //      ev1 = Evidence.preserve(conns)
    //      ev2 = Evidence.preserve(listeners)
    //  }
    // ----------------------------------------------------------------
    #[test]
    fn valid_02_network_snapshot() {
        let inv = investigation(
            "Network Snapshot",
            vec![
                assign("conns", None, module_call("Network", "connections", vec![], 2, 5), 2, 5),
                assign("listeners", None, module_call("Network", "listeners", vec![], 3, 5), 3, 5),
                assign("ev1", None, module_call("Evidence", "preserve", vec![var_expr("conns", 4, 35)], 4, 5), 4, 5),
                assign("ev2", None, module_call("Evidence", "preserve", vec![var_expr("listeners", 5, 35)], 5, 5), 5, 5),
            ],
        );

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&inv);
        assert!(result.is_ok(), "Valid script 2 (network snapshot) should pass: {:?}", result.err());
    }

    // ----------------------------------------------------------------
    //  ✅ VALID SCRIPT 3: Minimal — system info only
    //
    //  investigation "Host Baseline" {
    //      sys = System.info()
    //  }
    // ----------------------------------------------------------------
    #[test]
    fn valid_03_minimal_system_info() {
        let inv = investigation(
            "Host Baseline",
            vec![
                assign("sys", None, module_call("System", "info", vec![], 2, 5), 2, 5),
            ],
        );

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&inv);
        assert!(result.is_ok(), "Valid script 3 (minimal system info) should pass: {:?}", result.err());
    }

    // ----------------------------------------------------------------
    //  ✅ VALID SCRIPT 4: Evidence chain with verify
    //
    //  investigation "Evidence Chain" {
    //      procs = Process.collect()
    //      ev = Evidence.preserve(procs)
    //      ok = Evidence.verify(ev)
    //  }
    // ----------------------------------------------------------------
    #[test]
    fn valid_04_evidence_chain_with_verify() {
        let inv = investigation(
            "Evidence Chain",
            vec![
                assign("procs", None, module_call("Process", "collect", vec![], 2, 5), 2, 5),
                assign("ev", None, module_call("Evidence", "preserve", vec![var_expr("procs", 3, 35)], 3, 5), 3, 5),
                assign("ok", None, module_call("Evidence", "verify", vec![var_expr("ev", 4, 32)], 4, 5), 4, 5),
            ],
        );

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&inv);
        assert!(result.is_ok(), "Valid script 4 (evidence chain) should pass: {:?}", result.err());
    }

    // ----------------------------------------------------------------
    //  ✅ VALID SCRIPT 5: Complex filter with logical AND
    //
    //  investigation "Named and Sized" {
    //      procs = Process.collect()
    //      flagged = procs.filter(name == "cmd.exe" and memory > 500MB)
    //      ev = Evidence.preserve(flagged)
    //  }
    // ----------------------------------------------------------------
    #[test]
    fn valid_05_complex_filter_with_logical() {
        let filter_expr = Expr::BinaryLogical(BinaryLogical {
            left: Box::new(Expr::Comparison(Comparison {
                left: Box::new(Expr::Identifier(ident("name", 3, 30))),
                operator: ComparisonOp::EqualEqual,
                right: Box::new(Expr::Literal(Literal::String(str_lit("cmd.exe", 3, 38)))),
                span: span(3, 30),
            })),
            operator: LogicalBinaryOp::And,
            right: Box::new(Expr::Comparison(Comparison {
                left: Box::new(Expr::Identifier(ident("memory", 3, 56))),
                operator: ComparisonOp::Greater,
                right: Box::new(Expr::Literal(Literal::ByteSize {
                    bytes: 524_288_000,
                    raw: "500MB".to_string(),
                    span: span(3, 65),
                })),
                span: span(3, 56),
            })),
            span: span(3, 30),
        });

        let inv = investigation(
            "Named and Sized",
            vec![
                assign("procs", None, module_call("Process", "collect", vec![], 2, 5), 2, 5),
                assign("flagged", None, filter_call("procs", filter_expr, 3, 5), 3, 5),
                assign("ev", None, module_call("Evidence", "preserve", vec![var_expr("flagged", 4, 35)], 4, 5), 4, 5),
            ],
        );

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&inv);
        assert!(result.is_ok(), "Valid script 5 (complex filter) should pass: {:?}", result.err());
    }

    // ----------------------------------------------------------------
    //  ✅ REPRESENTATIVE PROGRAM TEST (from prompt & Handbook Section 7.6)
    //
    //  investigation "Endpoint Triage" {
    //      sys = System.info()
    //      processes = Process.collect()
    //      suspicious = processes.filter(name == "powershell.exe" and memory > 500MB)
    //      netconns = Network.connections()
    //      listeners = Network.listeners()
    //      ev = Evidence.preserve(suspicious)
    //      Evidence.verify(ev)
    //  }
    // ----------------------------------------------------------------
    #[test]
    fn test_representative_endpoint_triage() {
        let prog = Program {
            investigation: investigation(
                "Endpoint Triage",
                vec![
                    assign("sys", None, module_call("System", "info", vec![], 2, 5), 2, 5),
                    assign("processes", None, module_call("Process", "collect", vec![], 3, 5), 3, 5),
                    assign("suspicious", None, filter_call(
                        "processes",
                        Expr::BinaryLogical(BinaryLogical {
                            left: Box::new(Expr::Comparison(Comparison {
                                left: Box::new(Expr::Identifier(ident("name", 4, 35))),
                                operator: ComparisonOp::EqualEqual,
                                right: Box::new(Expr::Literal(Literal::String(str_lit("powershell.exe", 4, 43)))),
                                span: span(4, 40),
                            })),
                            operator: LogicalBinaryOp::And,
                            right: Box::new(Expr::Comparison(Comparison {
                                left: Box::new(Expr::Identifier(ident("memory", 4, 64))),
                                operator: ComparisonOp::Greater,
                                right: Box::new(Expr::Literal(Literal::ByteSize {
                                    bytes: 524_288_000,
                                    raw: "500MB".to_string(),
                                    span: span(4, 73),
                                })),
                                span: span(4, 71),
                            })),
                            span: span(4, 60),
                        }),
                        4, 5,
                    ), 4, 5),
                    assign("netconns", None, module_call("Network", "connections", vec![], 5, 5), 5, 5),
                    assign("listeners", None, module_call("Network", "listeners", vec![], 6, 5), 6, 5),
                    assign("ev", None, module_call("Evidence", "preserve", vec![var_expr("suspicious", 7, 28)], 7, 5), 7, 5),
                    Statement::ExpressionStatement(module_call("Evidence", "verify", vec![var_expr("ev", 8, 21)], 8, 5)),
                ],
            ),
        };

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze_program(&prog);
        assert!(result.is_ok(), "Representative endpoint triage should pass semantic analysis: {:?}", result.err());
    }

    // ================================================================
    //  ❌ INVALID SCRIPTS — each must be rejected with a clear error
    // ================================================================

    // ----------------------------------------------------------------
    //  ❌ INVALID 1: Unknown module "Filesystem" (not in allowlist)
    //
    //  investigation "Bad Module" {
    //      files = Filesystem.scan()
    //  }
    // ----------------------------------------------------------------
    #[test]
    fn invalid_01_unknown_module() {
        let inv = investigation(
            "Bad Module",
            vec![
                assign("files", None, module_call("Filesystem", "scan", vec![], 2, 5), 2, 5),
            ],
        );

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&inv);
        assert!(result.is_err(), "Should reject unknown module 'Filesystem'");
        let err = result.unwrap_err();
        match &err {
            SemanticError::UnknownModule { name, .. } => {
                assert_eq!(name, "Filesystem");
            }
            _ => panic!("Expected UnknownModule error, got: {}", err),
        }
        let msg = err.to_string();
        assert!(msg.contains("Filesystem"), "Error should name the bad module: {}", msg);
        assert!(msg.contains("System"), "Error should suggest valid modules: {}", msg);
    }

    // ----------------------------------------------------------------
    //  ❌ INVALID 2: Unknown function on known module (Process.scan)
    //
    //  investigation "Bad Function" {
    //      procs = Process.scan()
    //  }
    // ----------------------------------------------------------------
    #[test]
    fn invalid_02_unknown_function() {
        let inv = investigation(
            "Bad Function",
            vec![
                assign("procs", None, module_call("Process", "scan", vec![], 2, 5), 2, 5),
            ],
        );

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&inv);
        assert!(result.is_err(), "Should reject unknown function 'Process.scan'");
        let err = result.unwrap_err();
        match &err {
            SemanticError::UnknownFunction { module, function, .. } => {
                assert_eq!(module, "Process");
                assert_eq!(function, "scan");
            }
            _ => panic!("Expected UnknownFunction error, got: {}", err),
        }
        let msg = err.to_string();
        assert!(msg.contains("Process.scan"), "Error should identify the bad call: {}", msg);
        assert!(msg.contains("allowlist"), "Error should reference the allowlist: {}", msg);
    }

    // ----------------------------------------------------------------
    //  ❌ INVALID 3: Wrong arg count — System.info() takes 0, given 1
    //
    //  investigation "Too Many Args" {
    //      sys = System.info("extra")
    //  }
    // ----------------------------------------------------------------
    #[test]
    fn invalid_03_argument_count_mismatch() {
        let inv = investigation(
            "Too Many Args",
            vec![
                assign("sys", None, module_call(
                    "System", "info",
                    vec![Expr::Literal(Literal::String(str_lit("extra", 2, 22)))],
                    2, 5,
                ), 2, 5),
            ],
        );

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&inv);
        assert!(result.is_err(), "Should reject System.info with 1 arg");
        let err = result.unwrap_err();
        match &err {
            SemanticError::ArgumentCountMismatch { module, function, expected, actual, .. } => {
                assert_eq!(module, "System");
                assert_eq!(function, "info");
                assert_eq!(*expected, 0);
                assert_eq!(*actual, 1);
            }
            _ => panic!("Expected ArgumentCountMismatch error, got: {}", err),
        }
        let msg = err.to_string();
        assert!(msg.contains("expects 0"), "Error should state expected count: {}", msg);
        assert!(msg.contains("found 1"), "Error should state actual count: {}", msg);
    }

    // ----------------------------------------------------------------
    //  ❌ INVALID 4: Type mismatch — Evidence.preserve() requires
    //     ForensicArtifact, not a String literal
    //
    //  investigation "Wrong Type" {
    //      ev = Evidence.preserve("hello")
    //  }
    // ----------------------------------------------------------------
    #[test]
    fn invalid_04_type_mismatch_preserve_string() {
        let inv = investigation(
            "Wrong Type",
            vec![
                assign("ev", None, module_call(
                    "Evidence", "preserve",
                    vec![Expr::Literal(Literal::String(str_lit("hello", 2, 30)))],
                    2, 5,
                ), 2, 5),
            ],
        );

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&inv);
        assert!(result.is_err(), "Should reject Evidence.preserve(String)");
        let err = result.unwrap_err();
        match &err {
            SemanticError::TypeMismatch { module, function, expected, actual, .. } => {
                assert_eq!(module, "Evidence");
                assert_eq!(function, "preserve");
                assert!(expected.contains("ForensicArtifact"), "Expected should mention ForensicArtifact");
                assert_eq!(actual, "String");
            }
            _ => panic!("Expected TypeMismatch error, got: {}", err),
        }
    }

    // ----------------------------------------------------------------
    //  ❌ INVALID 5: Undefined variable reference
    //
    //  investigation "Undefined Var" {
    //      ev = Evidence.preserve(missing_var)
    //  }
    // ----------------------------------------------------------------
    #[test]
    fn invalid_05_undefined_variable() {
        let inv = investigation(
            "Undefined Var",
            vec![
                assign("ev", None, module_call(
                    "Evidence", "preserve",
                    vec![var_expr("missing_var", 2, 30)],
                    2, 5,
                ), 2, 5),
            ],
        );

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&inv);
        assert!(result.is_err(), "Should reject reference to undefined variable");
        let err = result.unwrap_err();
        match &err {
            SemanticError::UndefinedVariable { name, .. } => {
                assert_eq!(name, "missing_var");
            }
            _ => panic!("Expected UndefinedVariable error, got: {}", err),
        }
        let msg = err.to_string();
        assert!(msg.contains("missing_var"), "Error should name the undefined variable: {}", msg);
        assert!(msg.contains("line 2"), "Error should include line number: {}", msg);
    }
}