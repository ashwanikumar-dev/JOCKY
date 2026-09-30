// JOCKY Lexer — Compiler Frontend (M2)
//
// Tokenizer for the JOCKY forensic DSL.
// Converts JOCKY source text into a token stream.
//
// Reference: JOCKY Engineering Handbook §14.1
//   "source text → tokens ... separates lexical concerns (whitespace,
//    literals, keywords) from grammatical structure, keeping the parser simple."

pub mod token;
pub mod error;
pub mod lexer;

// Re-export public types for convenient access by downstream crates.
pub use token::{Token, TokenKind, Span};
pub use error::LexerError;
pub use lexer::Lexer;
