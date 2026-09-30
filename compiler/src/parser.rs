//! Grammar v0.1 Parser
//!
//! Parses Jocky forensic scripts into the AST defined in `ast.rs`.
//! This is the M2 boundary — the grammar may evolve but the parser
//! contract (source text → `ast::Script`) remains stable.

use crate::ast::{Argument, Expr, FunctionCall, Script, Span, Statement};
use thiserror::Error;

/// Errors that can occur during parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token at position {position}: {message}")]
    UnexpectedToken { position: usize, message: String },

    #[error("Unexpected end of input: {message}")]
    UnexpectedEof { message: String },

    #[error("Invalid string literal at position {position}")]
    InvalidString { position: usize },
}

/// The parser state, holding the source and current cursor position.
pub struct Parser<'a> {
    source: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given source text.
    pub fn new(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }

    /// Parse the full source into a `Script` AST.
    pub fn parse(&mut self) -> Result<Script, ParseError> {
        let mut statements = Vec::new();

        self.skip_whitespace();
        while self.pos < self.source.len() {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.skip_whitespace();
        }

        Ok(Script { statements })
    }

    /// Parse a single statement (currently only function calls).
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.pos;
        let name = self.parse_identifier()?;

        self.skip_whitespace();
        if self.peek_char() == Some('(') {
            let call = self.parse_function_call_rest(name, start)?;
            Ok(Statement::FunctionCall(call))
        } else {
            Err(ParseError::UnexpectedToken {
                position: self.pos,
                message: format!("Expected '(' after identifier '{}'", name),
            })
        }
    }

    /// Parse the arguments and closing paren of a function call.
    fn parse_function_call_rest(
        &mut self,
        name: String,
        start: usize,
    ) -> Result<FunctionCall, ParseError> {
        self.expect_char('(')?;
        let args = self.parse_arg_list()?;
        self.expect_char(')')?;

        Ok(FunctionCall {
            name,
            args,
            span: Span {
                start,
                end: self.pos,
            },
        })
    }

    /// Parse a comma-separated argument list.
    fn parse_arg_list(&mut self) -> Result<Vec<Argument>, ParseError> {
        let mut args = Vec::new();
        self.skip_whitespace();

        if self.peek_char() == Some(')') {
            return Ok(args);
        }

        loop {
            self.skip_whitespace();
            let arg = self.parse_argument()?;
            args.push(arg);
            self.skip_whitespace();

            if self.peek_char() == Some(',') {
                self.pos += 1; // consume comma
            } else {
                break;
            }
        }

        Ok(args)
    }

    /// Parse a single argument (positional expression for now).
    fn parse_argument(&mut self) -> Result<Argument, ParseError> {
        let expr = self.parse_expr()?;
        Ok(Argument::Positional(expr))
    }

    /// Parse an expression.
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.skip_whitespace();
        match self.peek_char() {
            Some('"') => self.parse_string_literal(),
            Some(c) if c.is_ascii_digit() => self.parse_int_literal(),
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                let start = self.pos;
                let ident = self.parse_identifier()?;

                // Check for boolean literals
                match ident.as_str() {
                    "true" => Ok(Expr::BoolLit(true)),
                    "false" => Ok(Expr::BoolLit(false)),
                    _ => {
                        self.skip_whitespace();
                        if self.peek_char() == Some('(') {
                            let call = self.parse_function_call_rest(ident, start)?;
                            Ok(Expr::Call(Box::new(call)))
                        } else {
                            Ok(Expr::Ident(ident))
                        }
                    }
                }
            }
            Some(c) => Err(ParseError::UnexpectedToken {
                position: self.pos,
                message: format!("Unexpected character '{}'", c),
            }),
            None => Err(ParseError::UnexpectedEof {
                message: "Expected expression".into(),
            }),
        }
    }

    /// Parse a string literal (double-quoted).
    fn parse_string_literal(&mut self) -> Result<Expr, ParseError> {
        let start = self.pos;
        self.pos += 1; // skip opening quote
        let mut value = String::new();

        loop {
            match self.peek_char() {
                Some('"') => {
                    self.pos += 1; // skip closing quote
                    return Ok(Expr::StringLit(value));
                }
                Some('\\') => {
                    self.pos += 1;
                    match self.peek_char() {
                        Some('n') => {
                            value.push('\n');
                            self.pos += 1;
                        }
                        Some('t') => {
                            value.push('\t');
                            self.pos += 1;
                        }
                        Some('\\') => {
                            value.push('\\');
                            self.pos += 1;
                        }
                        Some('"') => {
                            value.push('"');
                            self.pos += 1;
                        }
                        _ => {
                            return Err(ParseError::InvalidString { position: self.pos });
                        }
                    }
                }
                Some(c) => {
                    value.push(c);
                    self.pos += 1;
                }
                None => {
                    return Err(ParseError::InvalidString { position: start });
                }
            }
        }
    }

    /// Parse an integer literal.
    fn parse_int_literal(&mut self) -> Result<Expr, ParseError> {
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str = &self.source[start..self.pos];
        let value = num_str
            .parse::<i64>()
            .map_err(|_| ParseError::UnexpectedToken {
                position: start,
                message: format!("Invalid integer literal: {}", num_str),
            })?;
        Ok(Expr::IntLit(value))
    }

    /// Parse an identifier (alphanumeric + underscores).
    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(ParseError::UnexpectedToken {
                position: self.pos,
                message: "Expected identifier".into(),
            });
        }
        Ok(self.source[start..self.pos].to_string())
    }

    /// Peek at the current character without consuming it.
    fn peek_char(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    /// Consume and expect a specific character.
    fn expect_char(&mut self, expected: char) -> Result<(), ParseError> {
        self.skip_whitespace();
        match self.peek_char() {
            Some(c) if c == expected => {
                self.pos += c.len_utf8();
                Ok(())
            }
            Some(c) => Err(ParseError::UnexpectedToken {
                position: self.pos,
                message: format!("Expected '{}', found '{}'", expected, c),
            }),
            None => Err(ParseError::UnexpectedEof {
                message: format!("Expected '{}'", expected),
            }),
        }
    }

    /// Skip whitespace and comments.
    fn skip_whitespace(&mut self) {
        while self.pos < self.source.len() {
            let c = self.source.as_bytes()[self.pos];
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                self.pos += 1;
            } else if c == b'#' {
                // Line comment — skip to end of line
                while self.pos < self.source.len() && self.source.as_bytes()[self.pos] != b'\n' {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }
}

/// Convenience function: parse source text into a Script AST.
pub fn parse(source: &str) -> Result<Script, ParseError> {
    Parser::new(source).parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_call() {
        let script = parse("collect_processes()").unwrap();
        assert_eq!(script.statements.len(), 1);
        match &script.statements[0] {
            Statement::FunctionCall(call) => {
                assert_eq!(call.name, "collect_processes");
                assert!(call.args.is_empty());
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_parse_call_with_string_arg() {
        let script = parse(r#"hash_file("C:\\payload.exe")"#).unwrap();
        assert_eq!(script.statements.len(), 1);
        match &script.statements[0] {
            Statement::FunctionCall(call) => {
                assert_eq!(call.name, "hash_file");
                assert_eq!(call.args.len(), 1);
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_parse_multiple_statements() {
        let source = "collect_processes()\ncollect_network()";
        let script = parse(source).unwrap();
        assert_eq!(script.statements.len(), 2);
    }

    #[test]
    fn test_parse_with_comments() {
        let source = "# This is a comment\ncollect_processes()";
        let script = parse(source).unwrap();
        assert_eq!(script.statements.len(), 1);
    }
}
