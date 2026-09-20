# JOCKY Type System Specification

**Version:** 0.1  
**Status:** MVP / Contract Candidate  
**Owner:** M1 — Architecture & Language Design  
**Consumers:** M2 — Lexer/Parser/AST, M3 — Semantic Analysis/IR  
**Source:** JOCKY Grammar Specification v0.1 + Engineering Handbook

---

## 1. Purpose

This document defines the type system required by JOCKY Grammar v0.1.

The type system is used by semantic analysis to validate:

- variable assignments
- standard-library return values
- function arguments
- filter expressions
- field references
- operation compatibility
- evidence preservation and verification

Grammar validation determines whether a script is structurally valid.

Semantic analysis determines whether that structurally valid script is meaningful and type-correct.

---

## 2. Type Categories

JOCKY v0.1 contains two broad categories of types:

### 2.1 Primitive / Scalar Types

| Type | Meaning |
|---|---|
| `String` | Text value |
| `Integer` | Whole number |
| `Boolean` | `true` or `false` |
| `Float` | Decimal number |
| `Timestamp` | Point in time |
| `Hash` | Fixed-length integrity fingerprint |
| `IPAddress` | IPv4 or IPv6 address |
| `Port` | Network port number |

### 2.2 Forensic Domain Types

| Type | Meaning |
|---|---|
| `Process` | A single process record |
| `ProcessSet` | Collection of `Process` records |
| `NetworkConnection` | A single network connection record |
| `NetworkConnectionSet` | Collection of `NetworkConnection` records |
| `SystemInfo` | A single host/system information record |
| `Evidence` | A preserved evidence object |

---

## 3. MVP Type Set

The normative MVP semantic universe is:

```text
String
Integer
Boolean
Float
Timestamp
Hash
IPAddress
Port
Process
ProcessSet
NetworkConnection
Evidence