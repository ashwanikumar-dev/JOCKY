// JOCKY Recursive-Descent Parser — Compiler Frontend (M2)
//
// Parses a token stream into an AST following the JOCKY Grammar v0.1.
//
// Authoritative Specifications:
//   - JOCKY Grammar Specification v0.1 (M1 — September 2026)
//   - JOCKY 14-Day MVP Execution Plan §5 (Contract #1 & Contract #2)
//   - JOCKY Engineering Handbook §14.2
//
// Grammar v0.1:
//   Program             := InvestigationBlock ;
//   InvestigationBlock  := "investigation" StringLiteral "{" { Statement } "}" ;
//   Statement           := Assignment | ExpressionStatement ;
//   Assignment          := Identifier [ ":" TypeName ] "=" Expression ;
//   ExpressionStatement := Expression ;
//   TypeName            := Identifier ;
//   Expression          := LogicalOrExpr ;
//   LogicalOrExpr       := LogicalAndExpr { "or" LogicalAndExpr } ;
//   LogicalAndExpr      := NotExpr { "and" NotExpr } ;
//   NotExpr             := [ "not" ] ComparisonExpr ;
//   ComparisonExpr      := Postfix [ ComparisonOp Postfix ] ;
//   ComparisonOp        := "==" | "!=" | ">" | "<" | ">=" | "<=" ;
//   Postfix             := Primary { "." Identifier Arguments } ;
//   Primary             := Identifier | Literal | "(" Expression ")" ;
//   Arguments           := "(" [ Expression { "," Expression } ] ")" ;

use jocky_ast::*;
use jocky_lexer::{Span, Token, TokenKind};
use crate::error::ParseError;

/// The JOCKY recursive-descent syntax parser.
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    paren_depth: usize,
}

impl Parser {
    /// Creates a new parser for the given token stream.
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            paren_depth: 0,
        }
    }

    /// Parses the entire token stream into a Program AST.
    ///
    /// Grammar v0.1 §3.1, §3.2:
    ///   Program := InvestigationBlock ;
    /// Exactly one investigation block is permitted. Empty input and bare statements are rejected.
    pub fn parse(&mut self) -> Result<Program, ParseError> {
        self.skip_statement_newlines();

        if self.is_at_end() {
            let span = self.peek().span.clone();
            return Err(ParseError {
                message: "expected keyword 'investigation', found end of input".to_string(),
                span,
            });
        }

        if !self.check(TokenKind::Investigation) {
            let tok = self.peek();
            return Err(ParseError {
                message: format!("expected keyword 'investigation', found '{}'", tok.lexeme),
                span: tok.span.clone(),
            });
        }

        let investigation = self.parse_investigation()?;

        self.skip_statement_newlines();

        if !self.is_at_end() {
            let tok = self.peek();
            if tok.kind == TokenKind::Investigation {
                return Err(ParseError {
                    message: "multiple investigation blocks are not supported; a .jocky file must contain exactly one investigation block".to_string(),
                    span: tok.span.clone(),
                });
            } else {
                return Err(ParseError {
                    message: format!("unexpected token '{}' after investigation block", tok.lexeme),
                    span: tok.span.clone(),
                });
            }
        }

        Ok(Program { investigation })
    }

    // ── Investigation Block ──────────────────────────────────────────

    /// Parses an investigation block:
    ///   investigation "name" { statements... }
    fn parse_investigation(&mut self) -> Result<Investigation, ParseError> {
        let kw = self.expect(TokenKind::Investigation)?;
        let span = kw.span.clone();

        let name_tok = self.expect(TokenKind::StringLiteral)?;
        let name = StringLiteral {
            value: strip_quotes(&name_tok.lexeme),
            span: name_tok.span.clone(),
        };

        self.skip_statement_newlines();
        self.expect(TokenKind::LeftBrace)?;
        self.skip_statement_newlines();

        let mut body = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let stmt = self.parse_statement()?;
            body.push(stmt);
            self.skip_statement_newlines();
        }

        self.expect(TokenKind::RightBrace)?;

        Ok(Investigation { name, body, span })
    }

    // ── Statements ───────────────────────────────────────────────────

    /// Parses a statement inside an investigation block:
    ///   Statement := Assignment | ExpressionStatement ;
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        if self.check(TokenKind::Identifier) {
            if self.is_assignment_ahead() {
                self.parse_assignment()
            } else {
                self.parse_expression_statement()
            }
        } else if self.check(TokenKind::IntegerLiteral)
            || self.check(TokenKind::FloatLiteral)
            || self.check(TokenKind::StringLiteral)
            || self.check(TokenKind::ByteSizeLiteral)
            || self.check(TokenKind::True)
            || self.check(TokenKind::False)
        {
            let tok = self.peek();
            if self.peek_ahead(1).map(|t| t.kind == TokenKind::Equals).unwrap_or(false) {
                return Err(ParseError {
                    message: format!(
                        "invalid assignment target '{}'; assignment target must be an identifier",
                        tok.lexeme
                    ),
                    span: tok.span.clone(),
                });
            } else {
                self.parse_expression_statement()
            }
        } else {
            let tok = self.peek();
            if tok.lexeme == "+" || tok.lexeme == "-" || tok.lexeme == "*" || tok.lexeme == "/" {
                return Err(ParseError {
                    message: format!(
                        "unexpected operator '{}'; JOCKY does not support arithmetic operators",
                        tok.lexeme
                    ),
                    span: tok.span.clone(),
                });
            }
            Err(ParseError {
                message: format!(
                    "expected statement (assignment or expression), found '{}'",
                    tok.lexeme
                ),
                span: tok.span.clone(),
            })
        }
    }

    /// Disambiguation per Grammar v0.1 §15:
    /// Identifier followed by : or = is an Assignment; otherwise ExpressionStatement.
    fn is_assignment_ahead(&self) -> bool {
        let next_idx = self.current + 1;
        if next_idx < self.tokens.len() {
            let next_tok = &self.tokens[next_idx];
            next_tok.kind == TokenKind::Equals || next_tok.kind == TokenKind::Colon
        } else {
            false
        }
    }

    /// Parses an assignment:
    ///   Identifier [ ":" TypeName ] "=" Expression
    fn parse_assignment(&mut self) -> Result<Statement, ParseError> {
        let name_tok = self.expect(TokenKind::Identifier)?;
        let span = name_tok.span.clone();
        let target = Identifier {
            name: name_tok.lexeme.clone(),
            span: name_tok.span.clone(),
        };

        let type_annotation = if self.check_exact(TokenKind::Colon) {
            self.advance(); // consume ':'
            let type_tok = self.expect(TokenKind::Identifier)?;
            Some(Identifier {
                name: type_tok.lexeme.clone(),
                span: type_tok.span.clone(),
            })
        } else {
            None
        };

        self.expect(TokenKind::Equals)?;

        let value = self.parse_expr()?;

        self.ensure_statement_terminator()?;

        Ok(Statement::Assignment(Assignment {
            target,
            type_annotation,
            value,
            span,
        }))
    }

    /// Parses an expression statement:
    ///   ExpressionStatement := Expression ;
    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_expr()?;
        self.ensure_statement_terminator()?;
        Ok(Statement::ExpressionStatement(expr))
    }

    /// Checks that a statement is terminated by Newline, RightBrace, or Eof.
    fn ensure_statement_terminator(&self) -> Result<(), ParseError> {
        let tok = self.peek();
        if tok.kind == TokenKind::Newline || tok.kind == TokenKind::RightBrace || tok.kind == TokenKind::Eof {
            Ok(())
        } else if tok.lexeme == "+" || tok.lexeme == "-" || tok.lexeme == "*" || tok.lexeme == "/" {
            Err(ParseError {
                message: format!(
                    "unexpected operator '{}'; JOCKY does not support arithmetic operators",
                    tok.lexeme
                ),
                span: tok.span.clone(),
            })
        } else {
            Err(ParseError {
                message: format!("unexpected token '{}' after statement; expected newline", tok.lexeme),
                span: tok.span.clone(),
            })
        }
    }

    // ── Expression Precedence Hierarchy ──────────────────────────────

    /// Expression := LogicalOrExpr
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_logical_or()
    }

    /// LogicalOrExpr := LogicalAndExpr { "or" LogicalAndExpr } ;
    fn parse_logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(TokenKind::Or) {
            let op_span = self.previous().span.clone();
            let right = self.parse_logical_and()?;
            expr = Expr::BinaryLogical(BinaryLogical {
                left: Box::new(expr),
                operator: LogicalBinaryOp::Or,
                right: Box::new(right),
                span: op_span,
            });
        }

        Ok(expr)
    }

    /// LogicalAndExpr := NotExpr { "and" NotExpr } ;
    fn parse_logical_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_not()?;

        while self.match_token(TokenKind::And) {
            let op_span = self.previous().span.clone();
            let right = self.parse_not()?;
            expr = Expr::BinaryLogical(BinaryLogical {
                left: Box::new(expr),
                operator: LogicalBinaryOp::And,
                right: Box::new(right),
                span: op_span,
            });
        }

        Ok(expr)
    }

    /// NotExpr := [ "not" ] ComparisonExpr ;
    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(TokenKind::Not) {
            let op_span = self.previous().span.clone();
            let inner = self.parse_comparison()?;
            return Ok(Expr::UnaryLogical(UnaryLogical {
                operator: LogicalUnaryOp::Not,
                expr: Box::new(inner),
                span: op_span,
            }));
        }

        self.parse_comparison()
    }

    /// ComparisonExpr := Postfix [ ComparisonOp Postfix ] ;
    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_postfix()?;

        if let Some(op) = self.match_comparison_op() {
            let op_span = self.previous().span.clone();
            let right = self.parse_postfix()?;
            return Ok(Expr::Comparison(Comparison {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                span: op_span,
            }));
        }

        Ok(left)
    }

    /// Matches and consumes one of the 6 comparison operators:
    ///   ==, !=, >, <, >=, <=
    fn match_comparison_op(&mut self) -> Option<ComparisonOp> {
        if self.match_token(TokenKind::EqualEqual) {
            Some(ComparisonOp::EqualEqual)
        } else if self.match_token(TokenKind::BangEqual) {
            Some(ComparisonOp::BangEqual)
        } else if self.match_token(TokenKind::Greater) {
            Some(ComparisonOp::Greater)
        } else if self.match_token(TokenKind::Less) {
            Some(ComparisonOp::Less)
        } else if self.match_token(TokenKind::GreaterEqual) {
            Some(ComparisonOp::GreaterEqual)
        } else if self.match_token(TokenKind::LessEqual) {
            Some(ComparisonOp::LessEqual)
        } else {
            None
        }
    }

    /// Postfix := Primary { "." Identifier Arguments } ;
    /// Unified rule handling both standard-library calls and .filter(expr).
    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        while self.match_token(TokenKind::Dot) {
            let method_tok = self.expect(TokenKind::Identifier)?;
            let call_span = method_tok.span.clone();

            self.expect(TokenKind::LeftParen)?;
            self.paren_depth += 1;
            let args = self.parse_argument_list()?;
            self.expect(TokenKind::RightParen)?;
            self.paren_depth -= 1;

            if method_tok.lexeme == "filter" {
                if args.len() != 1 {
                    return Err(ParseError {
                        message: format!(
                            ".filter() requires exactly 1 condition argument, found {}",
                            args.len()
                        ),
                        span: call_span,
                    });
                }
                let condition = args.into_iter().next().unwrap();
                expr = Expr::Filter(Filter {
                    source: Box::new(expr),
                    condition: Box::new(condition),
                    span: call_span,
                });
            } else {
                expr = Expr::Call(Call {
                    object: Box::new(expr),
                    function: Identifier {
                        name: method_tok.lexeme,
                        span: method_tok.span,
                    },
                    arguments: args,
                    span: call_span,
                });
            }
        }

        Ok(expr)
    }

    /// Primary := Identifier | Literal | "(" Expression ")" ;
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let tok = self.peek();

        match tok.kind {
            TokenKind::Identifier => {
                let id_tok = self.advance().clone();
                Ok(Expr::Identifier(Identifier {
                    name: id_tok.lexeme,
                    span: id_tok.span,
                }))
            }
            TokenKind::StringLiteral => {
                let lit_tok = self.advance().clone();
                Ok(Expr::Literal(Literal::String(StringLiteral {
                    value: strip_quotes(&lit_tok.lexeme),
                    span: lit_tok.span,
                })))
            }
            TokenKind::IntegerLiteral => {
                let lit_tok = self.advance().clone();
                let value: i64 = lit_tok.lexeme.parse().map_err(|_| ParseError {
                    message: format!("invalid integer literal '{}'", lit_tok.lexeme),
                    span: lit_tok.span.clone(),
                })?;
                Ok(Expr::Literal(Literal::Integer {
                    value,
                    span: lit_tok.span,
                }))
            }
            TokenKind::FloatLiteral => {
                let lit_tok = self.advance().clone();
                let value: f64 = lit_tok.lexeme.parse().map_err(|_| ParseError {
                    message: format!("invalid float literal '{}'", lit_tok.lexeme),
                    span: lit_tok.span.clone(),
                })?;
                Ok(Expr::Literal(Literal::Float {
                    value,
                    span: lit_tok.span,
                }))
            }
            TokenKind::True => {
                let lit_tok = self.advance().clone();
                Ok(Expr::Literal(Literal::Boolean {
                    value: true,
                    span: lit_tok.span,
                }))
            }
            TokenKind::False => {
                let lit_tok = self.advance().clone();
                Ok(Expr::Literal(Literal::Boolean {
                    value: false,
                    span: lit_tok.span,
                }))
            }
            TokenKind::ByteSizeLiteral => {
                let lit_tok = self.advance().clone();
                let lit = parse_byte_size(&lit_tok.lexeme, lit_tok.span)?;
                Ok(Expr::Literal(lit))
            }
            TokenKind::LeftParen => {
                self.advance(); // consume '('
                self.paren_depth += 1;
                let inner = self.parse_expr()?;
                self.expect(TokenKind::RightParen)?;
                self.paren_depth -= 1;
                Ok(inner)
            }
            _ => {
                let tok = self.peek();
                if tok.lexeme == "+" || tok.lexeme == "-" || tok.lexeme == "*" || tok.lexeme == "/" {
                    Err(ParseError {
                        message: format!(
                            "unexpected operator '{}'; JOCKY does not support arithmetic operators",
                            tok.lexeme
                        ),
                        span: tok.span.clone(),
                    })
                } else {
                    Err(ParseError {
                        message: format!("expected expression, found '{}'", tok.lexeme),
                        span: tok.span.clone(),
                    })
                }
            }
        }
    }

    /// Arguments := "(" [ Expression { "," Expression } ] ")" ;
    fn parse_argument_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();

        if self.check(TokenKind::RightParen) {
            return Ok(args);
        }

        args.push(self.parse_expr()?);

        while self.match_token(TokenKind::Comma) {
            args.push(self.parse_expr()?);
        }

        Ok(args)
    }

    // ── Token Navigation Helpers ─────────────────────────────────────

    fn peek(&self) -> &Token {
        let idx = self.current_meaningful_index();
        &self.tokens[idx]
    }

    fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        let mut idx = self.current;
        let mut counted = 0;
        while idx < self.tokens.len() {
            if self.paren_depth > 0 && self.tokens[idx].kind == TokenKind::Newline {
                idx += 1;
                continue;
            }
            if counted == offset {
                return Some(&self.tokens[idx]);
            }
            counted += 1;
            idx += 1;
        }
        None
    }

    fn current_meaningful_index(&self) -> usize {
        let mut idx = self.current;
        if self.paren_depth > 0 {
            while idx < self.tokens.len() && self.tokens[idx].kind == TokenKind::Newline {
                idx += 1;
            }
        }
        if idx >= self.tokens.len() {
            self.tokens.len() - 1
        } else {
            idx
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn check_exact(&self, kind: TokenKind) -> bool {
        if self.current < self.tokens.len() {
            self.tokens[self.current].kind == kind
        } else {
            false
        }
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if self.paren_depth > 0 {
            while self.current < self.tokens.len() && self.tokens[self.current].kind == TokenKind::Newline {
                self.current += 1;
            }
        }
        let tok = &self.tokens[self.current];
        if tok.kind != TokenKind::Eof {
            self.current += 1;
        }
        tok
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        if self.paren_depth > 0 {
            while self.current < self.tokens.len() && self.tokens[self.current].kind == TokenKind::Newline {
                self.current += 1;
            }
        }
        let tok = self.tokens[self.current].clone();
        if tok.kind == kind {
            if tok.kind != TokenKind::Eof {
                self.current += 1;
            }
            Ok(tok)
        } else {
            if tok.lexeme == "+" || tok.lexeme == "-" || tok.lexeme == "*" || tok.lexeme == "/" {
                return Err(ParseError {
                    message: format!(
                        "unexpected operator '{}'; JOCKY does not support arithmetic operators",
                        tok.lexeme
                    ),
                    span: tok.span.clone(),
                });
            }
            Err(ParseError {
                message: format!("expected {:?}, found '{}' ({:?})", kind, tok.lexeme, tok.kind),
                span: tok.span.clone(),
            })
        }
    }

    fn skip_statement_newlines(&mut self) {
        while self.current < self.tokens.len() && self.tokens[self.current].kind == TokenKind::Newline {
            self.current += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
}

/// Helper function to parse a ByteSizeLiteral string into `Literal::ByteSize`.
fn parse_byte_size(lexeme: &str, span: Span) -> Result<Literal, ParseError> {
    if lexeme.len() < 3 {
        return Err(ParseError {
            message: format!("invalid byte-size literal '{}'", lexeme),
            span,
        });
    }
    let (digits_part, unit_part) = lexeme.split_at(lexeme.len() - 2);
    let count: u64 = digits_part.parse().map_err(|_| ParseError {
        message: format!("invalid number in byte-size literal '{}'", lexeme),
        span: span.clone(),
    })?;
    let multiplier: u64 = match unit_part {
        "KB" => 1024,
        "MB" => 1024 * 1024,
        "GB" => 1024 * 1024 * 1024,
        "TB" => 1024 * 1024 * 1024 * 1024,
        _ => {
            return Err(ParseError {
                message: format!("unknown unit '{}' in byte-size literal '{}'", unit_part, lexeme),
                span,
            });
        }
    };
    let bytes = count.checked_mul(multiplier).ok_or_else(|| ParseError {
        message: format!("byte-size literal '{}' overflows u64", lexeme),
        span: span.clone(),
    })?;
    Ok(Literal::ByteSize {
        bytes,
        raw: lexeme.to_string(),
        span,
    })
}

/// Strips surrounding double quotes from a string literal lexeme.
fn strip_quotes(s: &str) -> String {
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}
