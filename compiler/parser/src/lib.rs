// JOCKY Parser — Compiler Frontend (M2)
//
// Recursive-descent parser for the JOCKY forensic DSL.
// Converts a token stream into an AST.
//
// Handbook §14.2:
//   "Tokens → AST, following the JOCKY grammar."

pub mod error;
pub mod parser;

#[cfg(test)]
mod tests;

pub use error::ParseError;
pub use parser::Parser;
