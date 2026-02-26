# Spec: Issue #6037 - Add core invariants unit tests for data_layer_m11_closure_evidence

- Issue: #6037
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #5976

## Problem Statement
`data_layer_m11_closure_evidence` has baseline tests but lacks explicit coverage for multi-gate rejection aggregation semantics and deterministic reason-code ordering.

## Scope
In scope:
- Add direct unit tests for deterministic multi-failure aggregation ordering.
- Validate projected hardening/critical decisions are preserved in output.
- Validate acceptance marker remains exclusive to all-gates-satisfied path.

Out of scope:
- Changes to closure evidence API or reason code constants.
- Cross-module integration behavior changes.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: Multi-gate failures produce deterministic reason-code ordering (`hardening`, `critical_scenario`, `evidence_gap`).
- AC-2: Rejection report preserves projected hardening/critical decisions and gate booleans.
- AC-3: Acceptance report remains exclusive to all-gates-satisfied input.

## Conformance Cases
- C-01 (Conformance, AC-1): Input failing all gates returns rejection with exactly three reason codes in deterministic order.
- C-02 (Unit, AC-2): Rejection report carries non-go hardening and non-conformant critical decisions unchanged.
- C-03 (Regression, AC-3): Acceptance path emits only `m11_closure_accepted` with `Accepted` decision.

## Success Metrics / Observable Signals
- New M11 closure-evidence tests pass and cover deterministic multi-reason aggregation semantics.
- AC-to-test mapping is explicit in PR verification table.
