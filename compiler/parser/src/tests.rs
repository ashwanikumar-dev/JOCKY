// JOCKY Parser Tests — Compiler Frontend (M2)
//
// Comprehensive tests compliant with JOCKY Grammar Specification v0.1
// and Frozen MVP AST Contract.

#[cfg(test)]
mod tests {
    use jocky_ast::*;
    use jocky_lexer::{Lexer, Span, Token, TokenKind};
    use crate::{ParseError, Parser};

    /// Helper: lex + parse a JOCKY source string and return the Program AST.
    fn parse_source(source: &str) -> Result<Program, ParseError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|e| ParseError {
            message: e.message,
            span: e.span,
        })?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    // ══════════════════════════════════════════════════════════════════
    // VALID GRAMMAR v0.1 EXAMPLES
    // ══════════════════════════════════════════════════════════════════

    // ── Example 1: Investigation Block (Handbook §11.3, Grammar v0.1 §7.1)
    #[test]
    fn test_example_1_investigation_block() {
        let source = r#"investigation "Host Check" {
    system = System.info()
    processes = Process.collect()
    network = Network.connections()
}"#;
        let program = parse_source(source).expect("should parse");
        let inv = &program.investigation;

        assert_eq!(inv.name.value, "Host Check");
        assert_eq!(inv.body.len(), 3);

        // system = System.info()
        match &inv.body[0] {
            Statement::Assignment(a) => {
                assert_eq!(a.target.name, "system");
                assert!(a.type_annotation.is_none());
                match &a.value {
                    Expr::Call(call) => {
                        assert_eq!(call.function.name, "info");
                        match call.object.as_ref() {
                            Expr::Identifier(id) => assert_eq!(id.name, "System"),
                            other => panic!("expected Identifier(System), got {:?}", other),
                        }
                        assert!(call.arguments.is_empty());
                    }
                    other => panic!("expected Call, got {:?}", other),
                }
            }
            other => panic!("expected Assignment, got {:?}", other),
        }

        // processes = Process.collect()
        match &inv.body[1] {
            Statement::Assignment(a) => {
                assert_eq!(a.target.name, "processes");
                match &a.value {
                    Expr::Call(call) => assert_eq!(call.function.name, "collect"),
                    other => panic!("expected Call, got {:?}", other),
                }
            }
            other => panic!("expected Assignment, got {:?}", other),
        }

        // network = Network.connections()
        match &inv.body[2] {
            Statement::Assignment(a) => {
                assert_eq!(a.target.name, "network");
                match &a.value {
                    Expr::Call(call) => assert_eq!(call.function.name, "connections"),
                    other => panic!("expected Call, got {:?}", other),
                }
            }
            other => panic!("expected Assignment, got {:?}", other),
        }
    }

    // ── Example 2: Collect and Filter with ByteSizeLiteral (Grammar v0.1 §7.3)
    #[test]
    fn test_example_2_collect_and_filter() {
        let source = r#"investigation "Suspicious Memory Usage" {
    processes = Process.collect()
    suspicious = processes.filter(memory > 500MB)
}"#;
        let program = parse_source(source).expect("should parse");
        let inv = &program.investigation;
        assert_eq!(inv.body.len(), 2);

        // suspicious = processes.filter(memory > 500MB)
        match &inv.body[1] {
            Statement::Assignment(a) => {
                assert_eq!(a.target.name, "suspicious");
                match &a.value {
                    Expr::Filter(filter) => {
                        match filter.source.as_ref() {
                            Expr::Identifier(id) => assert_eq!(id.name, "processes"),
                            other => panic!("expected Identifier(processes), got {:?}", other),
                        }
                        match filter.condition.as_ref() {
                            Expr::Comparison(comp) => {
                                match comp.left.as_ref() {
                                    Expr::Identifier(id) => assert_eq!(id.name, "memory"),
                                    other => panic!("expected Identifier(memory), got {:?}", other),
                                }
                                assert_eq!(comp.operator, ComparisonOp::Greater);
                                match comp.right.as_ref() {
                                    Expr::Literal(Literal::ByteSize { bytes, raw, .. }) => {
                                        assert_eq!(*bytes, 500 * 1024 * 1024);
                                        assert_eq!(raw, "500MB");
                                    }
                                    other => panic!("expected ByteSizeLiteral, got {:?}", other),
                                }
                            }
                            other => panic!("expected Comparison, got {:?}", other),
                        }
                    }
                    other => panic!("expected Filter, got {:?}", other),
                }
            }
            other => panic!("expected Assignment, got {:?}", other),
        }
    }

    // ── Example 3: Typed Assignment and Standalone Calls (Grammar v0.1 §7.6, §16 T6)
    #[test]
    fn test_example_3_typed_assignment_and_standalone_calls() {
        let source = r#"investigation "Evidence Collection" {
    processes: ProcessSet = Process.collect()
    evidence = Evidence.preserve(processes)
    Evidence.verify(evidence)
}"#;
        let program = parse_source(source).expect("should parse");
        let inv = &program.investigation;
        assert_eq!(inv.body.len(), 3);

        // 1: Typed assignment
        match &inv.body[0] {
            Statement::Assignment(a) => {
                assert_eq!(a.target.name, "processes");
                assert_eq!(a.type_annotation.as_ref().unwrap().name, "ProcessSet");
            }
            other => panic!("expected Assignment, got {:?}", other),
        }

        // 2: Assigned call
        match &inv.body[1] {
            Statement::Assignment(a) => {
                assert_eq!(a.target.name, "evidence");
            }
            other => panic!("expected Assignment, got {:?}", other),
        }

        // 3: Standalone ExpressionStatement: Evidence.verify(evidence)
        match &inv.body[2] {
            Statement::ExpressionStatement(Expr::Call(call)) => {
                assert_eq!(call.function.name, "verify");
                assert_eq!(call.arguments.len(), 1);
                match &call.arguments[0] {
                    Expr::Identifier(id) => assert_eq!(id.name, "evidence"),
                    other => panic!("expected Identifier, got {:?}", other),
                }
            }
            other => panic!("expected ExpressionStatement(Call), got {:?}", other),
        }
    }

    // ── Chained .filter() — Critical Team Allocation case
    #[test]
    fn test_chained_collect_filter() {
        let source = r#"investigation "Chained Case" {
    processes = Process.collect().filter(pid > 1000)
}"#;
        let program = parse_source(source).expect("should parse");
        let inv = &program.investigation;
        assert_eq!(inv.body.len(), 1);

        match &inv.body[0] {
            Statement::Assignment(a) => {
                assert_eq!(a.target.name, "processes");
                match &a.value {
                    Expr::Filter(filter) => {
                        // Source is Call(Process.collect())
                        match filter.source.as_ref() {
                            Expr::Call(call) => {
                                assert_eq!(call.function.name, "collect");
                                assert!(call.arguments.is_empty());
                            }
                            other => panic!("expected Call, got {:?}", other),
                        }
                        // Condition is pid > 1000
                        match filter.condition.as_ref() {
                            Expr::Comparison(comp) => {
                                assert_eq!(comp.operator, ComparisonOp::Greater);
                                match comp.left.as_ref() {
                                    Expr::Identifier(id) => assert_eq!(id.name, "pid"),
                                    other => panic!("expected Identifier(pid), got {:?}", other),
                                }
                                match comp.right.as_ref() {
                                    Expr::Literal(Literal::Integer { value, .. }) => assert_eq!(*value, 1000),
                                    other => panic!("expected Integer(1000), got {:?}", other),
                                }
                            }
                            other => panic!("expected Comparison, got {:?}", other),
                        }
                    }
                    other => panic!("expected Filter, got {:?}", other),
                }
            }
            other => panic!("expected Assignment, got {:?}", other),
        }
    }

    // ── All 6 Comparison Operators
    #[test]
    fn test_all_six_comparison_operators() {
        let cases = [
            ("name == \"powershell.exe\"", ComparisonOp::EqualEqual),
            ("name != \"svchost.exe\"", ComparisonOp::BangEqual),
            ("memory > 500MB", ComparisonOp::Greater),
            ("pid < 1000", ComparisonOp::Less),
            ("port >= 80", ComparisonOp::GreaterEqual),
            ("port <= 443", ComparisonOp::LessEqual),
        ];

        for (expr_str, expected_op) in cases {
            let source = format!("investigation \"Test\" {{\n    f = procs.filter({})\n}}", expr_str);
            let program = parse_source(&source).expect("should parse");
            match &program.investigation.body[0] {
                Statement::Assignment(a) => match &a.value {
                    Expr::Filter(f) => match f.condition.as_ref() {
                        Expr::Comparison(c) => assert_eq!(c.operator, expected_op),
                        other => panic!("expected Comparison, got {:?}", other),
                    },
                    other => panic!("expected Filter, got {:?}", other),
                },
                other => panic!("expected Assignment, got {:?}", other),
            }
        }
    }

    // ── Logical Combinators: and, or, not (Grammar v0.1 §7.4, §7.6)
    #[test]
    fn test_filter_with_and_combinator() {
        let source = r#"investigation "Named and Sized" {
    suspicious = processes.filter(name == "powershell.exe" and memory > 500MB)
}"#;
        let program = parse_source(source).expect("should parse");
        match &program.investigation.body[0] {
            Statement::Assignment(a) => match &a.value {
                Expr::Filter(f) => match f.condition.as_ref() {
                    Expr::BinaryLogical(b) => {
                        assert_eq!(b.operator, LogicalBinaryOp::And);
                        match b.left.as_ref() {
                            Expr::Comparison(c) => assert_eq!(c.operator, ComparisonOp::EqualEqual),
                            other => panic!("expected Comparison on left, got {:?}", other),
                        }
                        match b.right.as_ref() {
                            Expr::Comparison(c) => assert_eq!(c.operator, ComparisonOp::Greater),
                            other => panic!("expected Comparison on right, got {:?}", other),
                        }
                    }
                    other => panic!("expected BinaryLogical, got {:?}", other),
                },
                other => panic!("expected Filter, got {:?}", other),
            },
            other => panic!("expected Assignment, got {:?}", other),
        }
    }

    #[test]
    fn test_filter_with_or_and_parentheses() {
        // Grammar v0.1 §7.6 realistic triage example
        let source = r#"investigation "Endpoint Triage" {
    suspicious = processes.filter(
        (name == "powershell.exe" or name == "cmd.exe") and memory > 500MB
    )
}"#;
        let program = parse_source(source).expect("should parse");
        match &program.investigation.body[0] {
            Statement::Assignment(a) => match &a.value {
                Expr::Filter(f) => match f.condition.as_ref() {
                    Expr::BinaryLogical(and_node) => {
                        assert_eq!(and_node.operator, LogicalBinaryOp::And);
                        // Left should be the parenthesized `or` expression
                        match and_node.left.as_ref() {
                            Expr::BinaryLogical(or_node) => {
                                assert_eq!(or_node.operator, LogicalBinaryOp::Or);
                            }
                            other => panic!("expected BinaryLogical(Or), got {:?}", other),
                        }
                        // Right is memory > 500MB
                        match and_node.right.as_ref() {
                            Expr::Comparison(comp) => {
                                assert_eq!(comp.operator, ComparisonOp::Greater);
                            }
                            other => panic!("expected Comparison, got {:?}", other),
                        }
                    }
                    other => panic!("expected BinaryLogical(And), got {:?}", other),
                },
                other => panic!("expected Filter, got {:?}", other),
            },
            other => panic!("expected Assignment, got {:?}", other),
        }
    }

    #[test]
    fn test_filter_with_not_negation() {
        let source = r#"investigation "Negation" {
    safe = processes.filter(not name == "malicious.exe")
}"#;
        let program = parse_source(source).expect("should parse");
        match &program.investigation.body[0] {
            Statement::Assignment(a) => match &a.value {
                Expr::Filter(f) => match f.condition.as_ref() {
                    Expr::UnaryLogical(u) => {
                        assert_eq!(u.operator, LogicalUnaryOp::Not);
                        match u.expr.as_ref() {
                            Expr::Comparison(c) => assert_eq!(c.operator, ComparisonOp::EqualEqual),
                            other => panic!("expected Comparison, got {:?}", other),
                        }
                    }
                    other => panic!("expected UnaryLogical, got {:?}", other),
                },
                other => panic!("expected Filter, got {:?}", other),
            },
            other => panic!("expected Assignment, got {:?}", other),
        }
    }

    // ── ByteSizeLiteral Units (KB, MB, GB, TB)
    #[test]
    fn test_byte_size_literal_units() {
        let units = [
            ("500KB", 500 * 1024),
            ("500MB", 500 * 1024 * 1024),
            ("2GB", 2 * 1024 * 1024 * 1024),
            ("1TB", 1024 * 1024 * 1024 * 1024),
        ];
        for (raw_str, expected_bytes) in units {
            let source = format!("investigation \"Test\" {{\n    s = procs.filter(size == {})\n}}", raw_str);
            let program = parse_source(&source).expect("should parse");
            match &program.investigation.body[0] {
                Statement::Assignment(a) => match &a.value {
                    Expr::Filter(f) => match f.condition.as_ref() {
                        Expr::Comparison(c) => match c.right.as_ref() {
                            Expr::Literal(Literal::ByteSize { bytes, raw, .. }) => {
                                assert_eq!(*bytes, expected_bytes);
                                assert_eq!(raw, raw_str);
                            }
                            other => panic!("expected ByteSize, got {:?}", other),
                        },
                        other => panic!("expected Comparison, got {:?}", other),
                    },
                    other => panic!("expected Filter, got {:?}", other),
                },
                other => panic!("expected Assignment, got {:?}", other),
            }
        }
    }

    // ── Comments and Literals
    #[test]
    fn test_comments_and_literals() {
        let source = r#"// Investigation for testing
investigation "Test" {
    // string, integer, float, boolean
    s = "hello \"world\""
    i = 42
    f = 3.14
    b = true
}"#;
        let program = parse_source(source).expect("should parse");
        assert_eq!(program.investigation.body.len(), 4);
    }

    // ══════════════════════════════════════════════════════════════════
    // MALFORMED SCRIPT REJECTION TESTS (Grammar v0.1 Compliance)
    // ══════════════════════════════════════════════════════════════════

    // ── Grammar v0.1 §16 T7: Bare statement rejected
    #[test]
    fn test_reject_bare_top_level_statement() {
        let source = "s = Process.collect()";
        let result = parse_source(source);
        assert!(result.is_err(), "bare statements outside investigation must be rejected");
        let err = result.unwrap_err();
        assert!(err.message.contains("expected keyword 'investigation'"));
    }

    // ── Grammar v0.1 §3.1: Empty input rejected
    #[test]
    fn test_reject_empty_input() {
        let result = parse_source("");
        assert!(result.is_err(), "empty input must be rejected");
        let err = result.unwrap_err();
        assert!(err.message.contains("expected keyword 'investigation'"));
    }

    // ── Grammar v0.1 §3.1: Multiple investigation blocks rejected
    #[test]
    fn test_reject_multiple_investigation_blocks() {
        let source = r#"investigation "First" {
    s = System.info()
}
investigation "Second" {
    p = Process.collect()
}"#;
        let result = parse_source(source);
        assert!(result.is_err(), "multiple investigation blocks must be rejected");
        let err = result.unwrap_err();
        assert!(err.message.contains("multiple investigation blocks are not supported"));
    }

    // ── Grammar v0.1 §16 T10: Invalid assignment target rejected
    #[test]
    fn test_reject_invalid_assignment_target() {
        let source = r#"investigation "Test" {
    5 = Process.collect()
}"#;
        let result = parse_source(source);
        assert!(result.is_err(), "numeric literal assignment target must be rejected");
        let err = result.unwrap_err();
        assert!(err.message.contains("invalid assignment target"));
    }

    // ── Grammar v0.1 §8 Invalid Example 9, §16 T11: Arithmetic operator rejected
    #[test]
    fn test_reject_arithmetic_operator_with_clear_message() {
        // Feed tokens directly or via source where an arithmetic operator is encountered
        let tokens = vec![
            Token { kind: TokenKind::Investigation, lexeme: "investigation".to_string(), span: Span { line: 1, column: 1 } },
            Token { kind: TokenKind::StringLiteral, lexeme: "\"X\"".to_string(), span: Span { line: 1, column: 15 } },
            Token { kind: TokenKind::LeftBrace, lexeme: "{".to_string(), span: Span { line: 1, column: 19 } },
            Token { kind: TokenKind::Newline, lexeme: "\n".to_string(), span: Span { line: 1, column: 20 } },
            Token { kind: TokenKind::Identifier, lexeme: "bad".to_string(), span: Span { line: 2, column: 5 } },
            Token { kind: TokenKind::Equals, lexeme: "=".to_string(), span: Span { line: 2, column: 9 } },
            Token { kind: TokenKind::Identifier, lexeme: "p".to_string(), span: Span { line: 2, column: 11 } },
            Token { kind: TokenKind::Identifier, lexeme: "+".to_string(), span: Span { line: 2, column: 13 } },
            Token { kind: TokenKind::IntegerLiteral, lexeme: "5".to_string(), span: Span { line: 2, column: 15 } },
            Token { kind: TokenKind::Newline, lexeme: "\n".to_string(), span: Span { line: 2, column: 16 } },
            Token { kind: TokenKind::RightBrace, lexeme: "}".to_string(), span: Span { line: 3, column: 1 } },
            Token { kind: TokenKind::Eof, lexeme: "".to_string(), span: Span { line: 3, column: 2 } },
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.message.contains("does not support arithmetic operators"),
            "error should mention arithmetic operators not supported: {}",
            err.message
        );
    }

    // ── Grammar v0.1 §16 T8: Unbalanced parentheses rejected
    #[test]
    fn test_reject_unbalanced_parentheses() {
        let source = "investigation \"X\" {\n    s = Process.collect(\n}";
        let result = parse_source(source);
        assert!(result.is_err(), "missing closing paren must be rejected");
    }

    // ── Missing closing brace rejected
    #[test]
    fn test_reject_missing_closing_brace() {
        let source = "investigation \"Host Check\" {\n    system = System.info()\n";
        let result = parse_source(source);
        assert!(result.is_err(), "missing closing brace must be rejected");
        let err = result.unwrap_err();
        assert!(err.message.contains("RightBrace"));
    }

    // ── Malformed filter condition rejected
    #[test]
    fn test_reject_malformed_filter_missing_operand() {
        let source = "investigation \"X\" {\n    suspicious = processes.filter(memory >)\n}";
        let result = parse_source(source);
        assert!(result.is_err(), "filter missing right operand must be rejected");
    }

    // ── Double dot in call rejected
    #[test]
    fn test_reject_double_dot_in_call() {
        let source = "investigation \"X\" {\n    system = System..info()\n}";
        let result = parse_source(source);
        assert!(result.is_err(), "double dot must be rejected");
    }

    // ── Error carries line and column info
    #[test]
    fn test_error_has_line_and_column() {
        let source = "investigation \"Test\" {\n    system = \n}";
        let result = parse_source(source);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.span.line, 2);
    }

    // ── Day 4 Audit Tests ────────────────────────────────────────────

    // 1. Reject filter with zero arguments: processes.filter()
    #[test]
    fn test_reject_filter_zero_arguments() {
        let source = r#"investigation "Test" {
    processes = Process.collect()
    suspicious = processes.filter()
}"#;
        let result = parse_source(source);
        assert!(result.is_err(), "filter with zero arguments must be rejected");
        let err = result.unwrap_err();
        assert!(err.message.contains(".filter() requires exactly 1 condition argument"));
    }

    // 2. Reject filter with multiple arguments: processes.filter(memory > 500MB, pid > 1000)
    #[test]
    fn test_reject_filter_multiple_arguments() {
        let source = r#"investigation "Test" {
    processes = Process.collect()
    suspicious = processes.filter(memory > 500MB, pid > 1000)
}"#;
        let result = parse_source(source);
        assert!(result.is_err(), "filter with multiple arguments must be rejected");
        let err = result.unwrap_err();
        assert!(err.message.contains(".filter() requires exactly 1 condition argument"));
    }

    // 3. Reject assignment with missing RHS: system =
    #[test]
    fn test_reject_assignment_missing_rhs() {
        let source = "investigation \"Test\" {\n    system =\n}";
        let result = parse_source(source);
        assert!(result.is_err(), "assignment with missing RHS must be rejected");
        let err = result.unwrap_err();
        assert!(err.message.contains("expected expression"));
    }

    // 4. Reject typed assignment with missing type: processes: = Process.collect()
    #[test]
    fn test_reject_typed_assignment_missing_type() {
        let source = r#"investigation "Test" {
    processes: = Process.collect()
}"#;
        let result = parse_source(source);
        assert!(result.is_err(), "typed assignment with missing type name must be rejected");
        let err = result.unwrap_err();
        assert!(err.message.contains("expected Identifier"));
    }

    // 5. Reject two statements on the same line: a = System.info() b = Process.collect()
    #[test]
    fn test_reject_two_statements_on_same_line() {
        let source = r#"investigation "Test" {
    a = System.info() b = Process.collect()
}"#;
        let result = parse_source(source);
        assert!(result.is_err(), "two statements on the same line must be rejected");
        let err = result.unwrap_err();
        assert!(err.message.contains("unexpected token 'b' after statement; expected newline"));
    }

    // 6. Reject trailing tokens after the investigation block: unexpected
    #[test]
    fn test_reject_trailing_tokens_after_investigation_block() {
        let source = r#"investigation "Test" {
    system = System.info()
}
unexpected"#;
        let result = parse_source(source);
        assert!(result.is_err(), "trailing tokens after investigation block must be rejected");
        let err = result.unwrap_err();
        assert!(err.message.contains("unexpected token 'unexpected' after investigation block"));
    }

    // 7. Add a full valid investigation covering all 7 documented MVP stdlib calls
    #[test]
    fn test_full_valid_investigation_all_7_mvp_stdlib_calls() {
        let source = r#"investigation "Full MVP Investigation" {
    sys = System.info()
    processes = Process.collect()
    suspicious = processes.filter(memory > 500MB and pid > 1000)
    netconns = Network.connections()
    listeners = Network.listeners()
    ev = Evidence.preserve(suspicious)
    verified = Evidence.verify(ev)
}"#;
        let program = parse_source(source).expect("full MVP investigation should parse successfully");
        let inv = &program.investigation;
        assert_eq!(inv.name.value, "Full MVP Investigation");
        assert_eq!(inv.body.len(), 7);

        // 1. System.info()
        match &inv.body[0] {
            Statement::Assignment(a) => {
                assert_eq!(a.target.name, "sys");
                match &a.value {
                    Expr::Call(c) => assert_eq!(c.function.name, "info"),
                    other => panic!("expected Call, got {:?}", other),
                }
            }
            other => panic!("expected Assignment, got {:?}", other),
        }

        // 2. Process.collect()
        match &inv.body[1] {
            Statement::Assignment(a) => {
                assert_eq!(a.target.name, "processes");
                match &a.value {
                    Expr::Call(c) => assert_eq!(c.function.name, "collect"),
                    other => panic!("expected Call, got {:?}", other),
                }
            }
            other => panic!("expected Assignment, got {:?}", other),
        }

        // 3. Process.filter()
        match &inv.body[2] {
            Statement::Assignment(a) => {
                assert_eq!(a.target.name, "suspicious");
                match &a.value {
                    Expr::Filter(f) => {
                        match f.condition.as_ref() {
                            Expr::BinaryLogical(b) => assert_eq!(b.operator, LogicalBinaryOp::And),
                            other => panic!("expected BinaryLogical, got {:?}", other),
                        }
                    }
                    other => panic!("expected Filter, got {:?}", other),
                }
            }
            other => panic!("expected Assignment, got {:?}", other),
        }

        // 4. Network.connections()
        match &inv.body[3] {
            Statement::Assignment(a) => {
                assert_eq!(a.target.name, "netconns");
                match &a.value {
                    Expr::Call(c) => assert_eq!(c.function.name, "connections"),
                    other => panic!("expected Call, got {:?}", other),
                }
            }
            other => panic!("expected Assignment, got {:?}", other),
        }

        // 5. Network.listeners()
        match &inv.body[4] {
            Statement::Assignment(a) => {
                assert_eq!(a.target.name, "listeners");
                match &a.value {
                    Expr::Call(c) => assert_eq!(c.function.name, "listeners"),
                    other => panic!("expected Call, got {:?}", other),
                }
            }
            other => panic!("expected Assignment, got {:?}", other),
        }

        // 6. Evidence.preserve()
        match &inv.body[5] {
            Statement::Assignment(a) => {
                assert_eq!(a.target.name, "ev");
                match &a.value {
                    Expr::Call(c) => assert_eq!(c.function.name, "preserve"),
                    other => panic!("expected Call, got {:?}", other),
                }
            }
            other => panic!("expected Assignment, got {:?}", other),
        }

        // 7. Evidence.verify()
        match &inv.body[6] {
            Statement::Assignment(a) => {
                assert_eq!(a.target.name, "verified");
                match &a.value {
                    Expr::Call(c) => assert_eq!(c.function.name, "verify"),
                    other => panic!("expected Call, got {:?}", other),
                }
            }
            other => panic!("expected Assignment, got {:?}", other),
        }
    }
}
