// JOCKY Token Definitions — Compiler Frontend (M2)
//
// Every token kind in this enum is derived from the authoritative JOCKY
// documentation (Engineering Handbook, 14-Day MVP Plan, Team Work Allocation).
//
// Classification key:
//   Cat 1 — Explicitly required by documented grammar rules
//   Cat 2 — Required by documented JOCKY syntax/examples
//   Cat 3 — Inference, included only with documented justification
//
// Reference documents:
//   - Handbook §11.2  Syntax and Grammar (conceptual)
//   - Handbook §11.3  Examples
//   - Handbook §11.4  Keywords
//   - Handbook §13    Type System
//   - Handbook §14.1  Lexer (token examples)
//   - MVP Plan §5     Interface Contracts (Grammar v0.1, Day 2 lexer spec)
//   - Team Allocation  M2 section (token definitions, .filter() operators)

/// The kind of a lexical token produced by the JOCKY lexer.
///
/// This enum covers exactly the lexical elements required by the JOCKY
/// forensic DSL grammar. Module names (System, Process, Network, etc.)
/// and function names (collect, filter, info, etc.) are tokenized as
/// `Identifier` — the semantic analysis stage (M3) validates which
/// module/function combinations are legal.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ── Keywords ─────────────────────────────────────────────────────
    //
    /// `investigation` — top-level investigation block keyword.
    /// Grammar v0.1 §2.2
    Investigation,

    /// `and` — logical conjunction in filter conditions.
    /// Grammar v0.1 §2.2, §3.2
    And,

    /// `or` — logical disjunction in filter conditions.
    /// Grammar v0.1 §2.2, §3.2
    Or,

    /// `not` — logical negation in filter conditions.
    /// Grammar v0.1 §2.2, §3.2
    Not,

    /// `true` — boolean literal keyword.
    /// Grammar v0.1 §2.2, §2.8.4
    True,

    /// `false` — boolean literal keyword.
    /// Grammar v0.1 §2.2, §2.8.4
    False,

    // ── Literals ─────────────────────────────────────────────────────
    //
    /// A double-quoted string literal, e.g. `"Host Check"`, `"powershell.exe"`.
    /// Grammar v0.1 §2.8.1
    StringLiteral,

    /// An integer literal, e.g. `500`, `1000`.
    /// Grammar v0.1 §2.8.2
    IntegerLiteral,

    /// A floating-point literal, e.g. `3.14`.
    /// Grammar v0.1 §2.8.3
    FloatLiteral,

    /// A byte-size literal with unit suffix KB, MB, GB, TB, e.g. `500MB`, `2GB`.
    /// Grammar v0.1 §2.8.5
    ByteSizeLiteral,

    // ── Identifiers ──────────────────────────────────────────────────
    //
    /// An identifier: variable names, module names, function names, field names.
    /// Grammar v0.1 §2.3
    Identifier,

    // ── Punctuation / Delimiters ─────────────────────────────────────
    //
    /// `{` — opens an investigation block.
    /// Grammar v0.1 §2.5
    LeftBrace,

    /// `}` — closes an investigation block.
    /// Grammar v0.1 §2.5
    RightBrace,

    /// `(` — opens a function argument list or grouping.
    /// Grammar v0.1 §2.5
    LeftParen,

    /// `)` — closes a function argument list or grouping.
    /// Grammar v0.1 §2.5
    RightParen,

    /// `.` — member / method access separator.
    /// Grammar v0.1 §2.4, §2.5
    Dot,

    /// `,` — argument separator.
    /// Grammar v0.1 §2.5
    Comma,

    /// `:` — type annotation separator.
    /// Grammar v0.1 §2.5
    Colon,

    // ── Assignment ───────────────────────────────────────────────────
    //
    /// `=` — variable assignment.
    /// Grammar v0.1 §2.4
    Equals,

    // ── Comparison Operators ─────────────────────────────────────────
    //
    /// `==` — equal to comparison operator.
    /// Grammar v0.1 §2.4, §6.3
    EqualEqual,

    /// `!=` — not equal to comparison operator.
    /// Grammar v0.1 §2.4, §6.3
    BangEqual,

    /// `>` — greater than comparison operator.
    /// Grammar v0.1 §2.4, §6.3
    Greater,

    /// `<` — less than comparison operator.
    /// Grammar v0.1 §2.4, §6.3
    Less,

    /// `>=` — greater than or equal to comparison operator.
    /// Grammar v0.1 §2.4, §6.3
    GreaterEqual,

    /// `<=` — less than or equal to comparison operator.
    /// Grammar v0.1 §2.4, §6.3
    LessEqual,

    // ── Special ──────────────────────────────────────────────────────
    //
    /// Newline character (`\n` or `\r\n`) — statement terminator.
    /// Grammar v0.1 §2.1, §3.3
    Newline,

    /// End of input — signals the token stream is complete.
    /// Grammar v0.1 §3.3
    Eof,
}

/// Source location of a token, for error reporting.
///
/// Both `line` and `column` are 1-based to match conventional editor
/// conventions and the error-reporting requirement from the Team
/// Allocation document: "Syntax error handling with line/column info".
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    /// 1-based line number in the source.
    pub line: usize,
    /// 1-based column number in the source.
    pub column: usize,
}

/// A single token produced by the JOCKY lexer.
///
/// Carries the token kind, the original source text (lexeme), and the
/// source location for error reporting.
///
/// Handbook §64 Interface Contract:
///   Lexer input:  .jocky source text
///   Lexer output: Token stream
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// What kind of token this is.
    pub kind: TokenKind,
    /// The original source text that produced this token.
    pub lexeme: String,
    /// Where in the source this token begins.
    pub span: Span,
}
