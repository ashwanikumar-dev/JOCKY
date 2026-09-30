# JOCKY Compiler Frontend (M2) — v1.0

## 1. Compiler Frontend Overview

The JOCKY compiler frontend processes JOCKY Domain-Specific Language (DSL) source text through a three-stage syntactic analysis pipeline:

```
JOCKY Source Text (.jocky)
           │
           ▼
     [ jocky-lexer ]        (Tokenization: keywords, identifiers, literals, operators, newlines)
           │
           ▼
      Token Stream
           │
           ▼
    [ jocky-parser ]        (Recursive-descent syntax parsing, operator precedence)
           │
           ▼
      AST v1.0 Root         (Abstract Syntax Tree: Program { investigation: Investigation })
```

The output of the M2 compiler frontend is the **AST (Abstract Syntax Tree)**. The frontend does not compile to bytecode, machine code, or intermediate representation; it hands the typed AST tree directly to downstream compilation stages.

---

## 2. M2 Responsibility

Member 2 (M2) owns exclusively the compiler frontend:
- **Lexer (`compiler/lexer/`)**: Character stream scanning, lexical error reporting with 1-based line/column spans, and token emission.
- **Parser (`compiler/parser/`)**: Recursive-descent grammar parsing, operator precedence resolution, syntactic validation, and syntax error generation.
- **AST (`compiler/ast/`)**: Rust data structures representing the syntactic tree, preserving 1-based source spans on all nodes, with Serde JSON serialization support.
- **Frontend Syntax & Error Handling**: Rejection of malformed syntax (unbalanced parentheses, unclosed blocks, bare statements, invalid assignment targets, unsupported operators).
- **Frontend Tests**: Crate-level unit tests and integration compliance suites under `tests/compiler/`.
- **Compiler Frontend Documentation**: Specification tracking and interface definitions for downstream consumers.

---

## 3. Explicit Non-Responsibilities

The M2 frontend explicitly does **NOT** implement, own, or execute any of the following architectural layers:
- **Semantic Analysis**: Type checking, module/function allowlist validation, field-existence resolution, and scope validation (owned by M3).
- **Forensic IR Generation & Validation**: Translation of AST nodes into Forensic IR instructions (`INVESTIGATION_BEGIN`, `COLLECT_*`, etc.) and IR schema validation (owned by M3).
- **Runtime & Dispatcher**: Endpoint execution engine and instruction dispatching (owned by M3).
- **Endpoint Agents**: Agent registration, task polling, packaging, and communication (owned by M4).
- **OS Collectors**: Windows and Ubuntu system APIs, process enumerators, network socket inspectors, and raw evidence acquisition (owned by M4).
- **Backend & Database**: FastAPI service, SQLite persistence, task management, and evidence storage (owned by M5).
- **Detection & Timeline**: Rule engine, risk scoring, event timestamp normalization, and timeline correlation (owned by M5).

---

## 4. Supported MVP Standard Library Calls (Frontend Syntax Recognition)

The frontend recognizes, parses, and represents the syntax of exactly seven MVP standard-library calls:

1. `System.info()`
2. `Process.collect()`
3. `Process.filter()`
4. `Network.connections()`
5. `Network.listeners()`
6. `Evidence.preserve()`
7. `Evidence.verify()`

### Frontend Scope Distinction:
- **Syntactic Representation**: The M2 frontend tokenizes these calls as `Identifier "." Identifier "(" Arguments? ")"`, builds `Expr::Call` or `Expr::Filter` nodes, and attaches arguments and source spans.
- **No Execution**: M2 does **NOT** execute forensic or OS collection. Calling `Process.collect()` in a script parses into an AST node; it does not enumerate OS processes during frontend compilation.

---

## 5. AST v1.0

The AST produced by the frontend adheres strictly to the frozen baseline contract:
- **Contract Reference**: `schemas/ast_v1_contract.md`
- **Node Types**: `Program`, `Investigation`, `Statement` (`Assignment`, `ExpressionStatement`), `Expr` (`Call`, `Filter`, `Comparison`, `BinaryLogical`, `UnaryLogical`, `Identifier`, `Literal`).
- **Source Spans**: Every AST node carries a 1-based `Span { line: usize, column: usize }` tracking its source origin.
- **JSON Serialization**: All AST types derive `serde::Serialize` and `serde::Deserialize` using remote derive (`SpanDef`) for cross-process interchange.

---

## 6. Implemented Grammar & Frontend Capabilities

The following syntax capabilities are confirmed by the current implementation and test suites:
- **Investigation Block**: Exactly one top-level `investigation "title" { ... }` block per program. Empty input, multiple blocks, or bare statements outside a block are rejected.
- **Variable Assignments**: Untyped assignments (`target = expr`) and typed assignments (`target: TypeName = expr`).
- **Function / Method Calls**: Member access calls `Module.function(args...)` and chained calls `expr.method(args...)`.
- **Collection Filtering**: Method-style filtering `.filter(condition)` requiring exactly 1 condition argument. Zero-argument and multi-argument `.filter()` calls are rejected.
- **Comparison Operators**: All six comparison operators: `==`, `!=`, `>`, `<`, `>=`, `<=`.
- **Logical Operators**: Left-associative `and`, `or`, and prefix `not`.
- **Precedence & Grouping**: Operator precedence (`or` < `and` < `not` < comparison < postfix) with parenthesized expressions `(` ... `)` to override precedence.
- **Byte-Size Literals**: Integer digits immediately followed by unit suffixes `KB`, `MB`, `GB`, `TB` (e.g. `500MB`), scanned as a single token and normalized to byte values in `Literal::ByteSize`.
- **Multiline Expressions**: Expressions inside parentheses or braces can span multiple lines; statement-terminating newlines are ignored within open parentheses.
- **String Escapes**: Double-quoted strings supporting literal quote `\"` and backslash `\\` escapes; unterminated strings or invalid escapes are rejected.
- **Syntax Rejection**: Clear syntax errors for unsupported arithmetic operators (`+`, `-`, `*`, `/`), illegal characters (e.g. `@`), and invalid assignment targets (e.g. `5 = ...`).

---

## 7. Testing & Verification

The compiler frontend is verified by four automated test suites using the GNU Rust toolchain (`stable-x86_64-pc-windows-gnu`):

| Test Suite | Location | Command | Actual Passing Tests |
|---|---|---|---|
| Compiler Compliance (T1–T13 + Day 7 Edge Cases) | `tests/compiler/frontend_tests.rs` | `cargo test --test compiler_compliance` | **19 passed; 0 failed** |
| Parser Unit & Diagnostic Tests | `compiler/parser/src/tests.rs` | `cargo test --manifest-path compiler/parser/Cargo.toml` | **27 passed; 0 failed** |
| Lexer Unit & Tokenization Tests | `compiler/lexer/src/lexer.rs` | `cargo test --manifest-path compiler/lexer/Cargo.toml` | **28 passed; 0 failed** |
| AST Construction & Serde Tests | `compiler/ast/src/lib.rs` | `cargo test --manifest-path compiler/ast/Cargo.toml` | **2 passed; 0 failed** |

**Total Verified Frontend Tests**: **76 passed; 0 failed; 0 warnings**.

---

## 8. v1.0 Freeze Statement

- **Status**: **FROZEN v1.0**.
- **Scope**: The M2 compiler frontend (`jocky-lexer`, `jocky-parser`, `jocky-ast`) is feature-complete for the 7-call MVP at the syntax/AST level.
- **Clarification**: This freeze certifies that the seven documented MVP calls and associated language constructs can be correctly tokenized, parsed, and represented in the frozen AST v1.0 schema. It does not certify or imply that backend execution or collector behavior is implemented.
- **Future Changes**: Any future modifications to `compiler/lexer/`, `compiler/parser/`, or `compiler/ast/` are strictly limited to critical bug fixes or formally approved grammar amendments agreed across all consuming members (M1, M2, M3).