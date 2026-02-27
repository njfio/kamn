# Spec: Issue #6039 - Add core invariants unit tests for data_layer_m11_hardening_readiness

- Issue: #6039
- Status: Implemented
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #5976

## Problem Statement
`data_layer_m11_hardening_readiness` currently lacks direct tests for readiness-reason precedence and aggregate missing/failing projection semantics across required scenarios.

## Scope
In scope:
- Add direct unit tests for operator readiness reason-code precedence and aggregate projection fields.
- Validate deterministic NoGo/Go outcomes for critical failures, incomplete required coverage, and fully passing required coverage.

Out of scope:
- Changes to readiness algorithm semantics.
- Cross-module integration behavior.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: Critical required failures take precedence and emit only `m11_blocking_critical_failure` reason.
- AC-2: Missing/incomplete required coverage without critical failures emits `m11_blocking_required_incomplete`.
- AC-3: Fully passing required coverage emits Go decision with `m11_operator_readiness_go`.

## Conformance Cases
- C-01 (Conformance, AC-1): One critical required failure with additional missing required scenario still yields critical-failure reason precedence.
- C-02 (Unit, AC-2): Required scenario incompleteness without critical failures yields incomplete-requirements reason and deterministic counts.
- C-03 (Regression, AC-3): All required scenarios passed yields Go with deterministic reason code.

## Success Metrics / Observable Signals
- New readiness precedence tests pass in `kamn-core`.
- AC-to-test mapping is explicit in PR verification table.
