// JOCKY Lexer — Compiler Frontend (M2)
//
// Tokenizes JOCKY forensic DSL source text into a token stream.
//
// Reference: Handbook §14.1
//   "source text → tokens ... a scanner over the character stream
//    matching token patterns (identifiers, keywords, operators, literals)."
//
// Day 2 spec (MVP Plan):
//   "Implement lexer: tokenize identifiers, strings, numbers, braces,
//    parens, dot, assignment, keywords"
//
// This lexer recognizes ONLY the lexical elements documented in the
// JOCKY Engineering Handbook and MVP Plan. It does NOT add features
// from general-purpose languages.

use crate::error::LexerError;
use crate::token::{Token, TokenKind, Span};

/// The JOCKY lexer. Converts source text into a stream of tokens.
///
/// Handbook §64 Interface Contract:
///   Input:  .jocky source text
///   Output: Token stream (Vec<Token>)
///   Contract: "Every valid .jocky file produces token stream without errors"
pub struct Lexer {
    /// The source text as a vector of characters for indexed access.
    source: Vec<char>,
    /// Current position in the source (index into `source`).
    current: usize,
    /// Current line number (1-based).
    line: usize,
    /// Current column number (1-based).
    column: usize,
}

impl Lexer {
    /// Creates a new lexer for the given JOCKY source text.
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenizes the entire source text and returns a vector of tokens.
    ///
    /// On success, the final token is always `TokenKind::Eof`.
    /// On failure, returns a `LexerError` with the source location
    /// of the unrecognized character.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        loop {
            // Skip whitespace (spaces and tabs).
            // Newlines are NOT skipped — they are emitted as tokens
            // for line tracking, consistent with the Day 1 TokenKind design.
            self.skip_whitespace();

            if self.is_at_end() {
                tokens.push(self.make_token(TokenKind::Eof, ""));
                break;
            }

            let token = self.scan_token()?;
            tokens.push(token);
        }

        Ok(tokens)
    }

    /// Scans a single token from the current position.
    fn scan_token(&mut self) -> Result<Token, LexerError> {
        let ch = self.peek();
        let span_start = self.current_span();

        match ch {
            // ── Newlines ─────────────────────────────────────────────
            '\n' => {
                let token = self.make_token(TokenKind::Newline, "\n");
                self.advance();
                self.line += 1;
                self.column = 1;
                Ok(token)
            }
            '\r' => {
                // Handle \r\n (Windows) and bare \r.
                self.advance();
                if !self.is_at_end() && self.peek() == '\n' {
                    let token = self.make_token_at(TokenKind::Newline, "\r\n", span_start);
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                    Ok(token)
                } else {
                    let token = self.make_token_at(TokenKind::Newline, "\r", span_start);
                    self.line += 1;
                    self.column = 1;
                    Ok(token)
                }
            }

            // ── Single-line comments ─────────────────────────────────
            // Handbook §13 shows `// ProcessSet + Integer is not defined`
            // Comments are consumed, not emitted as tokens.
            '/' if self.peek_next() == Some('/') => {
                self.skip_comment();
                // After skipping the comment, scan the next token.
                // If we're at the end, return Eof.
                self.skip_whitespace();
                if self.is_at_end() {
                    Ok(self.make_token(TokenKind::Eof, ""))
                } else {
                    self.scan_token()
                }
            }

            // ── Single-character punctuation / Delimiters ───────────
            // Grammar v0.1 §2.5
            '{' => Ok(self.single_char_token(TokenKind::LeftBrace)),
            '}' => Ok(self.single_char_token(TokenKind::RightBrace)),
            '(' => Ok(self.single_char_token(TokenKind::LeftParen)),
            ')' => Ok(self.single_char_token(TokenKind::RightParen)),
            '.' => Ok(self.single_char_token(TokenKind::Dot)),
            ',' => Ok(self.single_char_token(TokenKind::Comma)),
            ':' => Ok(self.single_char_token(TokenKind::Colon)),

            // ── Assignment and Comparison Operators ──────────────────
            // Grammar v0.1 §2.4: =, ==, !=, >, <, >=, <=
            '=' => {
                self.advance();
                if !self.is_at_end() && self.peek() == '=' {
                    self.advance();
                    Ok(self.make_token_at(TokenKind::EqualEqual, "==", span_start))
                } else {
                    Ok(self.make_token_at(TokenKind::Equals, "=", span_start))
                }
            }
            '!' => {
                self.advance();
                if !self.is_at_end() && self.peek() == '=' {
                    self.advance();
                    Ok(self.make_token_at(TokenKind::BangEqual, "!=", span_start))
                } else {
                    Err(LexerError {
                        message: "unrecognized character '!'; did you mean '!='?".to_string(),
                        span: span_start,
                    })
                }
            }
            '>' => {
                self.advance();
                if !self.is_at_end() && self.peek() == '=' {
                    self.advance();
                    Ok(self.make_token_at(TokenKind::GreaterEqual, ">=", span_start))
                } else {
                    Ok(self.make_token_at(TokenKind::Greater, ">", span_start))
                }
            }
            '<' => {
                self.advance();
                if !self.is_at_end() && self.peek() == '=' {
                    self.advance();
                    Ok(self.make_token_at(TokenKind::LessEqual, "<=", span_start))
                } else {
                    Ok(self.make_token_at(TokenKind::Less, "<", span_start))
                }
            }

            // ── String literals ──────────────────────────────────────
            // Grammar v0.1 §2.8.1: supports \" and \\ escapes; no multiline
            '"' => self.scan_string(),

            // ── Numeric & Byte-Size literals ─────────────────────────
            // Grammar v0.1 §2.8.2, §2.8.3, §2.8.5
            c if c.is_ascii_digit() => self.scan_number(),

            // ── Identifiers and keywords ─────────────────────────────
            // Grammar v0.1 §2.2, §2.3
            c if c.is_ascii_alphabetic() || c == '_' => self.scan_identifier(),

            // ── Unrecognized character ───────────────────────────────
            _ => {
                let ch = self.advance();
                Err(LexerError {
                    message: format!("unrecognized character '{}'", ch),
                    span: span_start,
                })
            }
        }
    }

    // ── Scanning helpers ─────────────────────────────────────────────

    /// Scans a double-quoted string literal.
    ///
    /// Grammar v0.1 §2.8.1:
    ///   StringLiteral := '"' StringChar* '"' ;
    ///   StringChar := any character except '"', backslash, or newline | '\' EscapedChar ;
    ///   EscapedChar := '"' | '\' ;
    ///
    /// Supports `\"` and `\\` escapes only. Reject invalid escapes.
    /// Strings may not span multiple lines.
    fn scan_string(&mut self) -> Result<Token, LexerError> {
        let start_span = self.current_span();
        let start = self.current;

        // Consume opening `"`.
        self.advance();

        while !self.is_at_end() {
            let ch = self.peek();
            if ch == '"' {
                // Consume closing `"`.
                self.advance();
                let lexeme: String = self.source[start..self.current].iter().collect();
                return Ok(self.make_token_at(TokenKind::StringLiteral, &lexeme, start_span));
            }
            if ch == '\n' || ch == '\r' {
                return Err(LexerError {
                    message: "unterminated string literal (hit newline)".to_string(),
                    span: start_span,
                });
            }
            if ch == '\\' {
                let esc_span = self.current_span();
                self.advance(); // consume '\'
                if self.is_at_end() {
                    return Err(LexerError {
                        message: "unterminated string literal (hit end of input)".to_string(),
                        span: start_span,
                    });
                }
                let esc = self.peek();
                if esc == '"' || esc == '\\' {
                    self.advance(); // consume escaped quote or backslash
                } else if esc == '\n' || esc == '\r' {
                    return Err(LexerError {
                        message: "unterminated string literal (hit newline)".to_string(),
                        span: start_span,
                    });
                } else {
                    return Err(LexerError {
                        message: format!(
                            "invalid escape sequence '\\{}' in string literal; only \\\" and \\\\ are supported",
                            esc
                        ),
                        span: esc_span,
                    });
                }
            } else {
                self.advance();
            }
        }

        Err(LexerError {
            message: "unterminated string literal (hit end of input)".to_string(),
            span: start_span,
        })
    }

    /// Scans a numeric literal (integer or float) or ByteSizeLiteral.
    ///
    /// Grammar v0.1:
    ///   IntegerLiteral  := Digit+ ;
    ///   FloatLiteral    := Digit+ "." Digit+ ;
    ///   ByteSizeLiteral := Digit+ ByteUnit ;
    ///   ByteUnit        := "KB" | "MB" | "GB" | "TB" ;
    fn scan_number(&mut self) -> Result<Token, LexerError> {
        let start_span = self.current_span();
        let start = self.current;

        // Consume leading digits.
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        // Check for float (decimal point followed by digits).
        if !self.is_at_end() && self.peek() == '.' {
            if let Some(next) = self.peek_at(self.current + 1) {
                if next.is_ascii_digit() {
                    self.advance(); // consume '.'
                    while !self.is_at_end() && self.peek().is_ascii_digit() {
                        self.advance();
                    }
                    let lexeme: String = self.source[start..self.current].iter().collect();
                    return Ok(self.make_token_at(TokenKind::FloatLiteral, &lexeme, start_span));
                } else if next.is_ascii_alphabetic() || next == '_' {
                    // e.g. `500.filter(...)` — dot is member access, not a decimal point.
                    let lexeme: String = self.source[start..self.current].iter().collect();
                    return Ok(self.make_token_at(TokenKind::IntegerLiteral, &lexeme, start_span));
                } else {
                    // Trailing decimal point without digits (e.g., `5. ` or `5.\n`).
                    // Grammar v0.1 §2.8.3, §9.1: lexical error.
                    self.advance(); // consume '.'
                    return Err(LexerError {
                        message: "malformed float literal: trailing decimal point with no digits".to_string(),
                        span: start_span,
                    });
                }
            } else {
                // Trailing decimal point at EOF (e.g. `5.`).
                self.advance(); // consume '.'
                return Err(LexerError {
                    message: "malformed float literal: trailing decimal point with no digits".to_string(),
                    span: start_span,
                });
            }
        }

        // Check for ByteSizeLiteral (digits immediately followed by KB, MB, GB, TB).
        // Grammar v0.1 §2.8.5: "No space is permitted between the digits and the unit."
        if !self.is_at_end() && (self.peek().is_ascii_alphabetic() || self.peek() == '_') {
            let ch0 = self.peek();
            let ch1 = self.peek_next();
            let is_byte_unit = match (ch0, ch1) {
                ('K', Some('B')) | ('M', Some('B')) | ('G', Some('B')) | ('T', Some('B')) => true,
                _ => false,
            };

            if is_byte_unit {
                let ch2 = self.peek_at(self.current + 2);
                let has_trailing_ident = match ch2 {
                    Some(c) => c.is_ascii_alphanumeric() || c == '_',
                    None => false,
                };
                if !has_trailing_ident {
                    self.advance(); // K / M / G / T
                    self.advance(); // B
                    let lexeme: String = self.source[start..self.current].iter().collect();
                    return Ok(self.make_token_at(TokenKind::ByteSizeLiteral, &lexeme, start_span));
                }
            }

            // Letters immediately follow digits, but not a valid ByteUnit (e.g., 500Mb, 500bytes, 500MBx).
            // Grammar v0.1 §9.1: malformed numeric or byte-size literal.
            while !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == '_') {
                self.advance();
            }
            let invalid_lexeme: String = self.source[start..self.current].iter().collect();
            return Err(LexerError {
                message: format!("malformed numeric or byte-size literal '{}'", invalid_lexeme),
                span: start_span,
            });
        }

        // Pure integer literal.
        let lexeme: String = self.source[start..self.current].iter().collect();
        Ok(self.make_token_at(TokenKind::IntegerLiteral, &lexeme, start_span))
    }

    /// Scans an identifier or keyword.
    ///
    /// Grammar v0.1 §2.2:
    ///   Keywords: investigation, and, or, not, true, false
    ///
    /// Module names (System, Process, Network, Evidence) and function names
    /// (collect, filter, info, etc.) remain as Identifier tokens.
    fn scan_identifier(&mut self) -> Result<Token, LexerError> {
        let start_span = self.current_span();
        let start = self.current;

        // Consume identifier characters.
        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme: String = self.source[start..self.current].iter().collect();

        // Check if the identifier is a reserved keyword (Grammar v0.1 §2.2).
        let kind = match lexeme.as_str() {
            "investigation" => TokenKind::Investigation,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            // Everything else is an identifier (module names, function
            // names, variable names, field names, type names, etc.).
            _ => TokenKind::Identifier,
        };

        Ok(self.make_token_at(kind, &lexeme, start_span))
    }

    // ── Character-level helpers ──────────────────────────────────────

    /// Returns the character at the current position without advancing.
    fn peek(&self) -> char {
        self.source[self.current]
    }

    /// Returns the character one position ahead, or None if at end.
    fn peek_next(&self) -> Option<char> {
        self.peek_at(self.current + 1)
    }

    /// Returns the character at a given index, or None if out of bounds.
    fn peek_at(&self, index: usize) -> Option<char> {
        if index < self.source.len() {
            Some(self.source[index])
        } else {
            None
        }
    }

    /// Advances past the current character and returns it.
    fn advance(&mut self) -> char {
        let ch = self.source[self.current];
        self.current += 1;
        self.column += 1;
        ch
    }

    /// Returns true if the lexer has consumed all source characters.
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    /// Skips horizontal whitespace (spaces and tabs only).
    /// Newlines are NOT skipped — they are emitted as Newline tokens.
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\t' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    /// Skips a single-line comment (from `//` to end of line).
    /// The comment content is consumed but not emitted as a token.
    fn skip_comment(&mut self) {
        // Consume the `//`.
        self.advance(); // first `/`
        self.advance(); // second `/`

        // Consume until newline or end of input.
        // The newline itself is NOT consumed here — it will be
        // picked up as a Newline token by the main scan loop.
        while !self.is_at_end() && self.peek() != '\n' && self.peek() != '\r' {
            self.advance();
        }
    }

    /// Returns the current source position as a Span.
    fn current_span(&self) -> Span {
        Span {
            line: self.line,
            column: self.column,
        }
    }

    /// Creates a token with the given kind and lexeme at the current span.
    fn make_token(&self, kind: TokenKind, lexeme: &str) -> Token {
        Token {
            kind,
            lexeme: lexeme.to_string(),
            span: self.current_span(),
        }
    }

    /// Creates a token with the given kind, lexeme, and explicit span.
    fn make_token_at(&self, kind: TokenKind, lexeme: &str, span: Span) -> Token {
        Token {
            kind,
            lexeme: lexeme.to_string(),
            span,
        }
    }

    /// Creates a single-character token, advancing past the character.
    fn single_char_token(&mut self, kind: TokenKind) -> Token {
        let span = self.current_span();
        let ch = self.advance();
        Token {
            kind,
            lexeme: ch.to_string(),
            span,
        }
    }
}

// ── Unit Tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenKind;

    /// Helper: tokenize source and return the token kinds (excluding Newline/Eof).
    fn token_kinds(source: &str) -> Vec<TokenKind> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("tokenization should succeed");
        tokens
            .into_iter()
            .filter(|t| t.kind != TokenKind::Newline && t.kind != TokenKind::Eof)
            .map(|t| t.kind)
            .collect()
    }

    /// Helper: tokenize source and return (kind, lexeme) pairs (excluding Newline/Eof).
    fn token_pairs(source: &str) -> Vec<(TokenKind, String)> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("tokenization should succeed");
        tokens
            .into_iter()
            .filter(|t| t.kind != TokenKind::Newline && t.kind != TokenKind::Eof)
            .map(|t| (t.kind, t.lexeme))
            .collect()
    }

    // ── Example Script 1 ─────────────────────────────────────────
    //
    // From Handbook §11.3:
    //   investigation "Host Check" {
    //     system = System.info()
    //     processes = Process.collect()
    //     network = Network.connections()
    //   }
    //
    // This tests: investigation keyword, string literal, braces,
    // identifiers (variable, module, function names), dot, parens,
    // assignment.

    #[test]
    fn test_example_1_investigation_block() {
        let source = r#"investigation "Host Check" {
    system = System.info()
    processes = Process.collect()
    network = Network.connections()
}"#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("should tokenize");

        // Collect non-whitespace, non-eof tokens for assertion.
        let meaningful: Vec<(TokenKind, &str)> = tokens
            .iter()
            .filter(|t| t.kind != TokenKind::Newline && t.kind != TokenKind::Eof)
            .map(|t| (t.kind.clone(), t.lexeme.as_str()))
            .collect();

        let expected: Vec<(TokenKind, &str)> = vec![
            // investigation "Host Check" {
            (TokenKind::Investigation, "investigation"),
            (TokenKind::StringLiteral, "\"Host Check\""),
            (TokenKind::LeftBrace, "{"),
            // system = System.info()
            (TokenKind::Identifier, "system"),
            (TokenKind::Equals, "="),
            (TokenKind::Identifier, "System"),
            (TokenKind::Dot, "."),
            (TokenKind::Identifier, "info"),
            (TokenKind::LeftParen, "("),
            (TokenKind::RightParen, ")"),
            // processes = Process.collect()
            (TokenKind::Identifier, "processes"),
            (TokenKind::Equals, "="),
            (TokenKind::Identifier, "Process"),
            (TokenKind::Dot, "."),
            (TokenKind::Identifier, "collect"),
            (TokenKind::LeftParen, "("),
            (TokenKind::RightParen, ")"),
            // network = Network.connections()
            (TokenKind::Identifier, "network"),
            (TokenKind::Equals, "="),
            (TokenKind::Identifier, "Network"),
            (TokenKind::Dot, "."),
            (TokenKind::Identifier, "connections"),
            (TokenKind::LeftParen, "("),
            (TokenKind::RightParen, ")"),
            // }
            (TokenKind::RightBrace, "}"),
        ];

        assert_eq!(meaningful.len(), expected.len(),
            "token count mismatch: got {}, expected {}", meaningful.len(), expected.len());

        for (i, (got, exp)) in meaningful.iter().zip(expected.iter()).enumerate() {
            assert_eq!(got, exp, "token {} mismatch", i);
        }
    }

    #[test]
    fn test_example_1_line_column_tracking() {
        let source = r#"investigation "Host Check" {
    system = System.info()
}"#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("should tokenize");

        // `investigation` is at line 1, column 1.
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.column, 1);
        assert_eq!(tokens[0].kind, TokenKind::Investigation);

        // `"Host Check"` is at line 1, column 15.
        assert_eq!(tokens[1].span.line, 1);
        assert_eq!(tokens[1].span.column, 15);
        assert_eq!(tokens[1].kind, TokenKind::StringLiteral);

        // `{` is at line 1, column 28.
        // "investigation" (13) + " " (1) + "\"Host Check\"" (12) + " " (1) + "{" starts at 28.
        assert_eq!(tokens[2].span.line, 1);
        assert_eq!(tokens[2].span.column, 28);
        assert_eq!(tokens[2].kind, TokenKind::LeftBrace);

        // Newline after `{`.
        assert_eq!(tokens[3].kind, TokenKind::Newline);
        assert_eq!(tokens[3].span.line, 1);

        // `system` is at line 2, column 5 (4 spaces indent + 1).
        assert_eq!(tokens[4].span.line, 2);
        assert_eq!(tokens[4].span.column, 5);
        assert_eq!(tokens[4].kind, TokenKind::Identifier);
        assert_eq!(tokens[4].lexeme, "system");
    }

    // ── Example Script 2 ─────────────────────────────────────────
    //
    // From Handbook §11.3 and Team Allocation:
    //   processes = Process.collect()
    //   suspicious = processes.filter(memory > 500)
    //
    // NOTE: The handbook shows `memory > 500MB` but `500MB` at the
    // lexical level is `IntegerLiteral(500) Identifier(MB)`. For
    // testing, we use `500` as the integer since the lexer tokenizes
    // character-by-character. The semantic layer handles units.
    //
    // This tests: identifiers, assignment, dot, parens, greater-than
    // operator, integer literal.

    #[test]
    fn test_example_2_collect_and_filter() {
        let source = "processes = Process.collect()\nsuspicious = processes.filter(memory > 500)";

        let pairs = token_pairs(source);

        let expected: Vec<(TokenKind, &str)> = vec![
            // processes = Process.collect()
            (TokenKind::Identifier, "processes"),
            (TokenKind::Equals, "="),
            (TokenKind::Identifier, "Process"),
            (TokenKind::Dot, "."),
            (TokenKind::Identifier, "collect"),
            (TokenKind::LeftParen, "("),
            (TokenKind::RightParen, ")"),
            // suspicious = processes.filter(memory > 500)
            (TokenKind::Identifier, "suspicious"),
            (TokenKind::Equals, "="),
            (TokenKind::Identifier, "processes"),
            (TokenKind::Dot, "."),
            (TokenKind::Identifier, "filter"),
            (TokenKind::LeftParen, "("),
            (TokenKind::Identifier, "memory"),
            (TokenKind::Greater, ">"),
            (TokenKind::IntegerLiteral, "500"),
            (TokenKind::RightParen, ")"),
        ];

        assert_eq!(pairs.len(), expected.len(),
            "token count mismatch: got {}, expected {}", pairs.len(), expected.len());

        for (i, (got, exp)) in pairs.iter().zip(expected.iter()).enumerate() {
            assert_eq!((got.0.clone(), got.1.as_str()), *exp, "token {} mismatch", i);
        }
    }

    #[test]
    fn test_example_2_unit_suffix_tokenization() {
        // Grammar v0.1 §2.8.5: `500MB` is scanned as a single, distinct ByteSizeLiteral token.
        let source = "processes.filter(memory > 500MB)";

        let pairs = token_pairs(source);

        let expected = vec![
            (TokenKind::Identifier, "processes".to_string()),
            (TokenKind::Dot, ".".to_string()),
            (TokenKind::Identifier, "filter".to_string()),
            (TokenKind::LeftParen, "(".to_string()),
            (TokenKind::Identifier, "memory".to_string()),
            (TokenKind::Greater, ">".to_string()),
            (TokenKind::ByteSizeLiteral, "500MB".to_string()),
            (TokenKind::RightParen, ")".to_string()),
        ];
        assert_eq!(pairs, expected);
    }

    // ── Example Script 3 ─────────────────────────────────────────
    //
    // From Handbook §13 (valid example) and the 7 MVP stdlib calls:
    //   processes: ProcessSet = Process.collect()
    //   suspicious: ProcessSet = processes.filter(memory > 500)
    //
    // This tests: type annotations (colon), identifiers as type names,
    // assignment, module.function() calls, filter with comparison.

    #[test]
    fn test_example_3_typed_assignment() {
        let source = "processes: ProcessSet = Process.collect()";

        let pairs = token_pairs(source);

        let expected: Vec<(TokenKind, &str)> = vec![
            (TokenKind::Identifier, "processes"),
            (TokenKind::Colon, ":"),
            (TokenKind::Identifier, "ProcessSet"),
            (TokenKind::Equals, "="),
            (TokenKind::Identifier, "Process"),
            (TokenKind::Dot, "."),
            (TokenKind::Identifier, "collect"),
            (TokenKind::LeftParen, "("),
            (TokenKind::RightParen, ")"),
        ];

        assert_eq!(pairs.len(), expected.len(),
            "token count mismatch: got {}, expected {}", pairs.len(), expected.len());

        for (i, (got, exp)) in pairs.iter().zip(expected.iter()).enumerate() {
            assert_eq!((got.0.clone(), got.1.as_str()), *exp, "token {} mismatch", i);
        }
    }

    #[test]
    fn test_example_3_full_typed_script() {
        // Full script combining typed assignments from Handbook §13.
        let source = r#"investigation "Evidence Collection" {
    processes: ProcessSet = Process.collect()
    evidence = Evidence.preserve()
    verified = Evidence.verify()
}"#;

        let kinds = token_kinds(source);

        // Verify the token kind sequence.
        let expected_kinds = vec![
            TokenKind::Investigation,    // investigation
            TokenKind::StringLiteral,    // "Evidence Collection"
            TokenKind::LeftBrace,        // {
            // processes: ProcessSet = Process.collect()
            TokenKind::Identifier,       // processes
            TokenKind::Colon,            // :
            TokenKind::Identifier,       // ProcessSet
            TokenKind::Equals,           // =
            TokenKind::Identifier,       // Process
            TokenKind::Dot,              // .
            TokenKind::Identifier,       // collect
            TokenKind::LeftParen,        // (
            TokenKind::RightParen,       // )
            // evidence = Evidence.preserve()
            TokenKind::Identifier,       // evidence
            TokenKind::Equals,           // =
            TokenKind::Identifier,       // Evidence
            TokenKind::Dot,              // .
            TokenKind::Identifier,       // preserve
            TokenKind::LeftParen,        // (
            TokenKind::RightParen,       // )
            // verified = Evidence.verify()
            TokenKind::Identifier,       // verified
            TokenKind::Equals,           // =
            TokenKind::Identifier,       // Evidence
            TokenKind::Dot,              // .
            TokenKind::Identifier,       // verify
            TokenKind::LeftParen,        // (
            TokenKind::RightParen,       // )
            // }
            TokenKind::RightBrace,       // }
        ];

        assert_eq!(kinds, expected_kinds);
    }

    // ── Additional lexer behavior tests ──────────────────────────

    #[test]
    fn test_eof_on_empty_input() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize().expect("should tokenize");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
    }

    #[test]
    fn test_boolean_literals() {
        let pairs = token_pairs("true false");
        assert_eq!(pairs, vec![
            (TokenKind::True, "true".to_string()),
            (TokenKind::False, "false".to_string()),
        ]);
    }

    #[test]
    fn test_integer_literal() {
        let pairs = token_pairs("42 0 12345");
        assert_eq!(pairs, vec![
            (TokenKind::IntegerLiteral, "42".to_string()),
            (TokenKind::IntegerLiteral, "0".to_string()),
            (TokenKind::IntegerLiteral, "12345".to_string()),
        ]);
    }

    #[test]
    fn test_float_literal() {
        let pairs = token_pairs("3.14 0.5");
        assert_eq!(pairs, vec![
            (TokenKind::FloatLiteral, "3.14".to_string()),
            (TokenKind::FloatLiteral, "0.5".to_string()),
        ]);
    }

    #[test]
    fn test_number_dot_not_float_when_followed_by_identifier() {
        // `500.filter` should be IntegerLiteral(500) Dot Identifier(filter),
        // NOT FloatLiteral.
        let pairs = token_pairs("500.filter");
        assert_eq!(pairs, vec![
            (TokenKind::IntegerLiteral, "500".to_string()),
            (TokenKind::Dot, ".".to_string()),
            (TokenKind::Identifier, "filter".to_string()),
        ]);
    }

    #[test]
    fn test_single_line_comment_skipped() {
        let source = "// this is a comment\nprocesses = Process.collect()";
        let pairs = token_pairs(source);
        // Comment is consumed; tokens start from `processes`.
        assert_eq!(pairs[0], (TokenKind::Identifier, "processes".to_string()));
    }

    #[test]
    fn test_comment_at_end_of_line() {
        let source = "system = System.info() // get system info";
        let kinds = token_kinds(source);
        assert_eq!(kinds, vec![
            TokenKind::Identifier,
            TokenKind::Equals,
            TokenKind::Identifier,
            TokenKind::Dot,
            TokenKind::Identifier,
            TokenKind::LeftParen,
            TokenKind::RightParen,
        ]);
    }

    #[test]
    fn test_unrecognized_character_error() {
        let mut lexer = Lexer::new("system @ info");
        let result = lexer.tokenize();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("unrecognized character"));
        assert!(err.message.contains('@'));
        assert_eq!(err.span.line, 1);
        assert_eq!(err.span.column, 8);
    }

    #[test]
    fn test_unterminated_string_error() {
        let mut lexer = Lexer::new("investigation \"Host Check");
        let result = lexer.tokenize();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("unterminated string"));
    }

    #[test]
    fn test_newline_tokens_emitted() {
        let source = "a\nb";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("should tokenize");
        // Expected: Identifier(a), Newline, Identifier(b), Eof
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::Newline);
        assert_eq!(tokens[2].kind, TokenKind::Identifier);
        assert_eq!(tokens[3].kind, TokenKind::Eof);
    }

    #[test]
    fn test_identifier_with_underscores_and_digits() {
        let pairs = token_pairs("event_id my_var2 _private");
        assert_eq!(pairs, vec![
            (TokenKind::Identifier, "event_id".to_string()),
            (TokenKind::Identifier, "my_var2".to_string()),
            (TokenKind::Identifier, "_private".to_string()),
        ]);
    }

    #[test]
    fn test_comma_in_function_args() {
        // Comma separator as implied by Contract #1 "module.function(args)".
        let pairs = token_pairs("Timeline.correlate(a, b)");
        assert_eq!(pairs, vec![
            (TokenKind::Identifier, "Timeline".to_string()),
            (TokenKind::Dot, ".".to_string()),
            (TokenKind::Identifier, "correlate".to_string()),
            (TokenKind::LeftParen, "(".to_string()),
            (TokenKind::Identifier, "a".to_string()),
            (TokenKind::Comma, ",".to_string()),
            (TokenKind::Identifier, "b".to_string()),
            (TokenKind::RightParen, ")".to_string()),
        ]);
    }

    #[test]
    fn test_lexer_reference_example_from_handbook() {
        // Handbook §14.1: Process.collect() conceptually becomes:
        //   IDENTIFIER(Process) DOT IDENTIFIER(collect) LPAREN RPAREN
        let pairs = token_pairs("Process.collect()");
        assert_eq!(pairs, vec![
            (TokenKind::Identifier, "Process".to_string()),
            (TokenKind::Dot, ".".to_string()),
            (TokenKind::Identifier, "collect".to_string()),
            (TokenKind::LeftParen, "(".to_string()),
            (TokenKind::RightParen, ")".to_string()),
        ]);
    }

    #[test]
    fn test_all_six_comparison_operators() {
        // Grammar v0.1 §2.4, §6.3: ==, !=, >, <, >=, <=
        let pairs = token_pairs("== != > < >= <=");
        assert_eq!(pairs, vec![
            (TokenKind::EqualEqual, "==".to_string()),
            (TokenKind::BangEqual, "!=".to_string()),
            (TokenKind::Greater, ">".to_string()),
            (TokenKind::Less, "<".to_string()),
            (TokenKind::GreaterEqual, ">=".to_string()),
            (TokenKind::LessEqual, "<=".to_string()),
        ]);
    }

    #[test]
    fn test_assignment_vs_equality() {
        // Grammar v0.1 §2.4: '=' is assignment, '==' is equality comparison
        let assign_pairs = token_pairs("x = 10");
        assert_eq!(assign_pairs, vec![
            (TokenKind::Identifier, "x".to_string()),
            (TokenKind::Equals, "=".to_string()),
            (TokenKind::IntegerLiteral, "10".to_string()),
        ]);

        let eq_pairs = token_pairs("x == 10");
        assert_eq!(eq_pairs, vec![
            (TokenKind::Identifier, "x".to_string()),
            (TokenKind::EqualEqual, "==".to_string()),
            (TokenKind::IntegerLiteral, "10".to_string()),
        ]);
    }

    #[test]
    fn test_logical_keywords_and_or_not() {
        // Grammar v0.1 §2.2: and, or, not are reserved keywords
        let pairs = token_pairs("and or not");
        assert_eq!(pairs, vec![
            (TokenKind::And, "and".to_string()),
            (TokenKind::Or, "or".to_string()),
            (TokenKind::Not, "not".to_string()),
        ]);
    }

    #[test]
    fn test_byte_size_literals_all_units() {
        // Grammar v0.1 §2.8.5: KB, MB, GB, TB
        let pairs = token_pairs("500KB 500MB 2GB 1TB");
        assert_eq!(pairs, vec![
            (TokenKind::ByteSizeLiteral, "500KB".to_string()),
            (TokenKind::ByteSizeLiteral, "500MB".to_string()),
            (TokenKind::ByteSizeLiteral, "2GB".to_string()),
            (TokenKind::ByteSizeLiteral, "1TB".to_string()),
        ]);
    }

    #[test]
    fn test_valid_escaped_strings() {
        // Grammar v0.1 §2.8.1: supports \" and \\
        let source = r#""quote: \" hello" "backslash: \\""#;
        let pairs = token_pairs(source);
        assert_eq!(pairs, vec![
            (TokenKind::StringLiteral, r#""quote: \" hello""#.to_string()),
            (TokenKind::StringLiteral, r#""backslash: \\""#.to_string()),
        ]);
    }

    #[test]
    fn test_invalid_string_escape_rejected() {
        // Grammar v0.1 §2.8.1: only \" and \\ are supported; \n or \t is a lexical error
        let mut lexer = Lexer::new(r#""hello \n world""#);
        let result = lexer.tokenize();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("invalid escape sequence"));
    }

    #[test]
    fn test_malformed_byte_size_literal_wrong_case() {
        // Grammar v0.1 §2.8.5, §9.1: 500Mb with wrong case is a lexical error
        let mut lexer = Lexer::new("500Mb");
        let result = lexer.tokenize();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("malformed numeric or byte-size literal"));
    }

    #[test]
    fn test_malformed_float_trailing_dot() {
        // Grammar v0.1 §2.8.3, §9.1: 5. with no trailing digit is a lexical error
        let mut lexer = Lexer::new("5. ");
        let result = lexer.tokenize();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("malformed float literal"));
    }

    #[test]
    fn test_bang_without_equal_rejected() {
        let mut lexer = Lexer::new("!");
        let result = lexer.tokenize();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("unrecognized character '!'"));
    }
}
