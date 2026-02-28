# Issue 6250 Spec

Status: Reviewed
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6246

## Problem Statement
Current shell/workflow/template surface remains larger than Rust surface (baseline audit: shell LOC `248615`, Rust LOC `237231`, ratio `1.0480`). This increases maintenance overhead and weakens typed test ownership.

## Scope
In scope:
- Migrate high-value lane logic from shell/python/workflow wrappers into Rust integration tests/utilities.
- Reduce shell/workflow/template LOC and increase Rust test/utility ownership.
- Enforce ratio non-regression with deterministic CI policy checks.

Out of scope:
- Rewriting every script in one pass.
- Runtime feature changes unrelated to lane ownership.

## Acceptance Criteria
- AC-1: Measured shell/workflow/template to Rust LOC ratio is reduced below `1.0`.
- AC-2: At least one high-frequency CI lane path is migrated from shell/python logic into Rust test or utility ownership.
- AC-3: CI enforces a fail-closed ratio non-regression policy with explicit reason codes.
- AC-4: Issue closure records required shell-surface DoD markers with measured actual deltas.

## Conformance Cases
- C-01 (AC-1, Conformance): Ratio measurement command/report shows `< 1.0` after changes.
- C-02 (AC-2, Integration): Migrated lane path executes through Rust-owned test/utilities and passes in CI.
- C-03 (AC-3, Regression): Policy gate fails when ratio regresses or measurement artifacts are missing.
- C-04 (AC-4, Functional): Closure artifacts include actual shell/rust delta fields and target status.

## Success Metrics
- Shell-to-Rust ratio crosses below parity and remains enforced by policy.
- CI lane logic ownership shifts toward Rust integration tests.
