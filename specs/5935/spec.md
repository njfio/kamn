# Spec: Issue #5935 - Task: Eliminate high-impact code duplication classes (JSON parser, helper utilities, digest/path helpers)

- Issue: #5935
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5919

## Problem Statement
Multiple duplicated helpers and parsers increase bug surface and inconsistency risk.

## Scope
In scope:
- Create shared utilities and migrate duplicate call sites (including RFC 8259-compliant JSON string parsing).

Out of scope:
- Behavioral changes outside existing helper contracts.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Duplicated parser/helper implementations are replaced with canonical shared modules.
- AC-2: JSON parser paths support escaped unicode and reject malformed payloads consistently.
- AC-3: Regression tests prevent reintroduction of duplicate helper classes.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify Duplicated parser/helper implementations are replaced with canonical shared modules.
- C-02 (Functional, AC-2): Verify JSON parser paths support escaped unicode and reject malformed payloads consistently.
- C-03 (Functional, AC-3): Verify Regression tests prevent reintroduction of duplicate helper classes.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: shared helper/parser test suites
- Functional: call-site parity checks
- Integration: crates consuming shared utilities remain green
- Regression: duplicate inventory checks
- Performance: parser utility overhead non-regression

## Dependencies
- #5919

