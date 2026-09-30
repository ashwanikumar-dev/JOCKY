// JOCKY Lexer Error — Compiler Frontend (M2)
//
// Error types for lexer failures, with source location information.
//
// Team Allocation M2: "Syntax error handling with line/column info"

use crate::Span;
use std::fmt;

/// An error produced by the JOCKY lexer when it encounters invalid input.
#[derive(Debug, Clone, PartialEq)]
pub struct LexerError {
    /// Human-readable description of the error.
    pub message: String,
    /// Where in the source the error occurred.
    pub span: Span,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lexer error at line {}, column {}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

impl std::error::Error for LexerError {}
