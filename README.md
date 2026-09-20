# JOCKY

> A compiler-driven forensic DSL and distributed investigation platform.

JOCKY is a cross-platform digital forensics platform built around a domain-specific language (DSL) for describing forensic investigations.

Instead of writing OS-specific commands or scripts, an investigator describes **what forensic information is required** using JOCKY. The compiler converts that investigation into a validated **Forensic Intermediate Representation (IR)**, which is then executed by authorized JOCKY agents through OS-specific collectors.

Collected evidence is normalized while preserving its original provenance, integrity, and machine context. A central backend stores the evidence and performs detection, timeline generation, correlation, auditing, and reporting.

---

## Table of Contents

- [Problem Statement](#problem-statement)
- [Proposed Solution](#proposed-solution)
- [Core Idea](#core-idea)
- [Key Capabilities](#key-capabilities)
- [System Architecture](#system-architecture)
- [End-to-End Workflow](#end-to-end-workflow)
- [JOCKY Language](#jocky-language)
- [Compiler Architecture](#compiler-architecture)
- [Forensic IR](#forensic-ir)
- [Runtime, Agent and Collector Model](#runtime-agent-and-collector-model)
- [Cross-Platform Execution](#cross-platform-execution)
- [Evidence Lifecycle](#evidence-lifecycle)
- [Evidence Integrity and Provenance](#evidence-integrity-and-provenance)
- [Detection and Forensic Intelligence](#detection-and-forensic-intelligence)
- [Timeline and Correlation](#timeline-and-correlation)
- [Distributed Architecture](#distributed-architecture)
- [Backend Architecture](#backend-architecture)
- [Dashboard](#dashboard)
- [Security Architecture](#security-architecture)
- [Repository Structure](#repository-structure)
- [Shared Contracts](#shared-contracts)
- [Technology Stack](#technology-stack)
- [MVP Scope](#mvp-scope)
- [Current Development Status](#current-development-status)
- [Team Ownership](#team-ownership)
- [Development Workflow](#development-workflow)
- [Contract and Versioning Rules](#contract-and-versioning-rules)
- [Local Development Setup](#local-development-setup)
- [Running the Backend](#running-the-backend)
- [Testing Strategy](#testing-strategy)
- [Integration Testing](#integration-testing)
- [Demo Workflow](#demo-workflow)
- [Deployment Model](#deployment-model)
- [Security and Research Scope](#security-and-research-scope)
- [Known Limitations](#known-limitations)
- [Future Roadmap](#future-roadmap)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

---

## Problem Statement

**SIH Problem Statement 26148**

> Creation of scripts/functions with new programming language to commence Computer & Network forensic analysis without triggering security solutions.

The project is intended for authorized computer and network forensic investigation and security-research environments.

JOCKY approaches the problem through a compiler-driven forensic DSL, a controlled Forensic IR, distributed endpoint agents, OS-specific collectors, evidence preservation, and centralized forensic intelligence.

---

## Proposed Solution

JOCKY introduces a domain-specific language designed specifically for forensic investigation.

The investigator describes the required forensic operation using JOCKY instead of directly writing operating-system-specific commands.

The system then:

1. Parses the JOCKY investigation.
2. Builds an Abstract Syntax Tree (AST).
3. Performs semantic and policy validation.
4. Converts the validated investigation into Forensic IR.
5. Validates the IR again before execution.
6. Creates an authorized task for the target machine.
7. Sends the task to the corresponding JOCKY Agent.
8. The agent executes only approved forensic operations.
9. OS-specific collectors acquire the required information.
10. Raw evidence is preserved.
11. Evidence is normalized into a common representation.
12. Provenance and SHA-256 integrity information are attached.
13. Evidence is transferred to the central backend.
14. The backend stores and verifies the evidence.
15. Detection and timeline processing are performed.
16. Findings, alerts, timelines and audit information are exposed through the dashboard.

The central architectural idea is:

> **Investigator intent → controlled representation → controlled execution → attributable evidence**

---

## Core Idea

JOCKY separates **what the investigator wants to investigate** from **how the target operating system performs the collection**.

For example:

```text
processes = Process.collect()
```

## SYSTEM ARCHITECTURE

                                JOCKY FORENSIC PLATFORM

 ┌─────────────────────────────────────────────────────────────────────────────┐
 │                         INVESTIGATION / CLIENT LAYER                        │
 │                                                                             │
 │   Investigator                                                            │
 │        │                                                                    │
 │        ▼                                                                    │
 │   JOCKY Script ────────────────────────────────────────────────────────┐    │
 │                                                                         │    │
 └─────────────────────────────────────────────────────────────────────────┼────┘
                                                                           │
                                                                           ▼
 ┌─────────────────────────────────────────────────────────────────────────────┐
 │                           COMPILER LAYER                                    │
 │                                                                             │
 │   Lexer → Parser → AST → Semantic Analysis → Forensic IR → IR Validator   │
 │                                                                             │
 └──────────────────────────────────────────┬──────────────────────────────────┘
                                            │
                                            ▼
 ┌─────────────────────────────────────────────────────────────────────────────┐
 │                         CENTRAL PLATFORM                                   │
 │                                                                             │
 │  ┌────────────────┐   ┌────────────────┐   ┌──────────────────────────┐    │
 │  │ Task Manager   │   │ Agent Manager  │   │ Investigation Management │    │
 │  └────────────────┘   └────────────────┘   └──────────────────────────┘    │
 │                                                                             │
 │  ┌────────────────┐   ┌────────────────┐   ┌──────────────────────────┐    │
 │  │ Evidence API   │   │ Evidence Store │   │ Audit / Custody          │    │
 │  └────────────────┘   └────────────────┘   └──────────────────────────┘    │
 │                                                                             │
 └──────────────┬──────────────────────────────────────┬───────────────────────┘
                │                                      │
                │ Control Plane                        │ Evidence / Results
                ▼                                      ▲
 ┌────────────────────────────┐          ┌────────────────────────────────────┐
 │       WINDOWS DOMAIN       │          │          FORENSIC INTELLIGENCE     │
 │                            │          │                                    │
 │  JOCKY Agent               │          │  Evidence                          │
 │       │                    │          │      │                             │
 │       ▼                    │          │      ├── Detection                 │
 │  Runtime / Dispatcher     │          │      ├── Findings                   │
 │       │                    │          │      ├── Alerts                     │
 │       ▼                    │          │      ├── Timeline                   │
 │  OS Collectors            │          │      └── Correlation                │
 │       │                    │          │                                    │
 │       ▼                    │          └────────────────────────────────────┘
 │  Windows OS               │
 └────────────────────────────┘
                │
                │
 ┌────────────────────────────┐
 │       UBUNTU DOMAIN        │
 │                            │
 │  JOCKY Agent               │
 │       │                    │
 │       ▼                    │
 │  Runtime / Dispatcher     │
 │       │                    │
 │       ▼                    │
 │  OS Collectors            │
 │       │                    │
 │       ▼                    │
 │  Ubuntu Linux              │
 └────────────────────────────┘

            



























            