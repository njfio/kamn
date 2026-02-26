# Spec: Issue #6011 - Production expect() census non-regression gate

- Issue: #6011
- Status: Implemented
- Type: task
- Priority: P1
- Area: qa
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26

## Problem Statement
The repository has no deterministic production-scope `expect()` non-regression gate, so panic-surface growth can slip into CI without an explicit budget contract.

## Scope
In scope:
- Add production-scope `expect()` census test fixture(s) and policy fixture(s).
- Add a fail-closed gate test that compares current production `expect()` count vs baseline.
- Emit deterministic reason-code output to make CI failures actionable.

Out of scope:
- Converting all current `expect()` usage in this issue.
- Full AST-level semantic classification of every `expect()` call site.

## Risk Level
`medium`

## Acceptance Criteria
- AC-1: `kamn-core` has a deterministic test that computes production-scope `expect()` count.
- AC-2: Test fails if current count exceeds baseline threshold.
- AC-3: Baseline and threshold fixtures are versioned and validated by schema markers.

## Conformance Cases
- C-01 (Unit, AC-1): census test computes a non-negative count from tracked production files.
- C-02 (Functional, AC-2): gate returns `NO-GO` with reason code when count exceeds allowed delta.
- C-03 (Regression, AC-3): fixture schema markers mismatch fails closed.

## Success Metrics / Observable Signals
- CI has an explicit production `expect()` growth ratchet.
- Audit can reference deterministic output fields (`reason_taxonomy_version`, `reason_codes_csv`, `reason_code`).
