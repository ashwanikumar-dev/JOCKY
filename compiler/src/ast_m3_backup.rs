use serde::{Deserialize, Serialize};

/// Source code location tracking for error reporting per Section 10
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash,Serialize, Deserialize)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// Root node: A JOCKY script is exactly one investigation block
/// EBNF: InvestigationBlock := "investigation" StringLiteral "{" { Statement } "}";
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Investigation {
    pub title: String,
    pub body: Vec<Statement>,
    pub span: Span,
}

/// A statement within an investigation block
/// EBNF: Statement := Assignment | ExpressionStatement;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Assignment(Assignment),
    ExpressionStatement(Expression),
}

/// Variable binding statement
/// EBNF: Assignment := Identifier [ ":" TypeName ] "=" Expression;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub target: Identifier,
    pub type_annotation: Option<Identifier>, // None if inferred, Some if explicitly declared
    pub value: Expression,
    pub span: Span,
}

/// General expression tree representing precedence chain (or < and < not < comparison < postfix)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    LogicalOr {
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
    LogicalAnd {
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
    LogicalNot {
        operand: Box<Expression>,
        span: Span,
    },
    Comparison(Comparison),
    Postfix(Postfix),
    Primary(Primary),
}

/// Comparison expression
/// EBNF: ComparisonExpr := Postfix [ ComparisonOp Postfix ];
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comparison {
    pub left: Postfix,
    pub op: ComparisonOp,
    pub right: Postfix,
    pub span: Span,
}

/// The 6 comparison operators supported in Grammar v0.1 Section 2.4
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    Equal,              // ==
    NotEqual,           // !=
    GreaterThan,        // >
    LessThan,           // <
    GreaterThanOrEqual, // >=
    LessThanOrEqual,    // <=
}

/// Postfix chain for stdlib calls and method-style filter chaining
/// EBNF: Postfix := Primary { "." Identifier Arguments };
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Postfix {
    pub base: Primary,
    pub calls: Vec<CallStep>,
    pub span: Span,
}

/// Represents each "." Identifier "(" [args] ")" in a call chain
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallStep {
    pub function: Identifier,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

/// Primary atom
/// EBNF: Primary := Identifier | Literal | "(" Expression ")";
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Primary {
    Identifier(Identifier),
    Literal(Literal),
    Grouped(Box<Expression>),
}

/// Plain identifier
/// EBNF: Identifier := IdentStart IdentChar*;
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Identifier {
    pub name: String,
    pub span: Span,
}

impl Identifier {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

/// Literal kinds supported in Grammar v0.1 Section 2.8
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    String {
        value: String,
        span: Span,
    },
    Integer {
        value: u64,
        span: Span,
    },
    Float {
        value: f64,
        span: Span,
    },
    Boolean {
        value: bool,
        span: Span,
    },
    ByteSize {
        raw_unit: String,     // e.g., "500MB"
        bytes: u64,           // normalized byte count: 524,288,000
        span: Span,
    },
}
