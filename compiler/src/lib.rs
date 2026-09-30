//! JOCKY Compiler Library
//!
//! End-to-end compiler pipeline:
//!
//! JOCKY source
//!   → M2 Lexer
//!   → M2 Parser
//!   → M3 Semantic Analysis
//!   → M3 Forensic IR Generation
//!   → M3 IR Validation

pub mod ast;
pub mod ir;
pub mod semantic;
pub mod validator;

use ir::{ForensicIrDocument, IrGenerator};
use semantic::SemanticAnalyzer;
use thiserror::Error;
use validator::IrValidator;

use jocky_lexer::{Lexer, LexerError};
use jocky_parser::{ParseError, Parser};

/// Errors produced by the end-to-end compiler pipeline.
#[derive(Debug, Error)]
pub enum CompileError {
    #[error("lexer error: {0}")]
    Lexer(#[from] LexerError),

    #[error("parser error: {0}")]
    Parser(#[from] ParseError),

    #[error("semantic analysis error: {0}")]
    Semantic(String),

    #[error("IR generation error: {0}")]
    IrGeneration(#[from] ir::IrGenError),

    #[error("IR validation error: {0}")]
    IrValidation(#[from] validator::IrValidationError),
}

/// Compile a JOCKY source script into validated Forensic IR.
///
/// Pipeline:
/// `source → tokens → AST → semantic validation → IR → IR validation`
pub fn compile(
    source: &str,
    investigation_id: impl Into<String>,
) -> Result<ForensicIrDocument, CompileError> {
    // ------------------------------------------------------------
    // 1. M2 Lexer
    // ------------------------------------------------------------
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    // ------------------------------------------------------------
    // 2. M2 Parser
    // ------------------------------------------------------------
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    // ------------------------------------------------------------
    // 3. M3 Semantic Analysis
    // ------------------------------------------------------------
    let mut semantic = SemanticAnalyzer::new();

    semantic
        .analyze_program(&program)
        .map_err(|err| CompileError::Semantic(err.to_string()))?;

    // ------------------------------------------------------------
    // 4. M3 IR Generation
    // ------------------------------------------------------------
    let generator = IrGenerator::new(investigation_id);

    let document = generator.generate(&program)?;

    // ------------------------------------------------------------
    // 5. M3 IR Validation
    // ------------------------------------------------------------
    IrValidator::validate(&document)?;

    Ok(document)
}