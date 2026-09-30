//! JOCKY AST bridge.
//!
//! M2 owns the authoritative AST contract.
//! M3 consumes that AST for semantic analysis and IR generation.

pub use jocky_ast::*;
pub use jocky_lexer::Span;