// JOCKY Parser Error — Compiler Frontend (M2)

use jocky_lexer::Span;
use std::fmt;

/// An error produced by the JOCKY parser when it encounters invalid syntax.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// Human-readable description of the error.
    pub message: String,
    /// Where in the source the error occurred.
    pub span: Span,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at line {}, column {}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}
