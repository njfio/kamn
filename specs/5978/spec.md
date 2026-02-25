# Spec: Issue #5978 - Task: Enforce governance/runtime test-ratio gate and consolidate duplicate doc-contract suites

- Issue: #5978
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-25)
- Type: task
- Priority: P1
- Area: qa
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-25
- Parent: #5975

## Problem Statement
Governance/doc-contract suites remain structurally heavy. We need deterministic ratio telemetry and enforceable non-regression thresholds.

## Scope
In scope:
- Implement ratio telemetry + policy check in CI.
- Consolidate one high-duplication doc-contract family.

Out of scope:
- Full governance framework redesign.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Governance/runtime ratio telemetry artifact generated in CI.
- AC-2: Policy gate fails closed on threshold breach.
- AC-3: First consolidation wave reduces duplication without behavioral-test regressions.

## Conformance Cases
- C-01 (Conformance, AC-1): Telemetry artifact schema and required keys validated.
- C-02 (Regression, AC-2): Tampered/threshold-breach fixtures fail gate deterministically.
- C-03 (Functional, AC-3): Consolidated suite keeps required behavior checks passing.

## Success Metrics / Observable Signals
- CI emits ratio telemetry and enforcement status every run.
- Reduced duplicate suite surface in first remediation wave.
