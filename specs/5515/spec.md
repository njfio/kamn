# Issue #5515 Spec - Spec Status Normalization for Merged Telemetry Task

- Status: Accepted
- Issue: #5515
- Parent: #3812
- Milestone: R50.23 Spec status normalization for merged telemetry task

## Problem Statement
Issue `#5513` is merged/closed but `specs/5513/spec.md` still reports `Status: Accepted`, violating lifecycle closure contract expectations.

## Scope
In scope:
- Update `specs/5513/spec.md` status line to `Implemented`.
- Extend lifecycle docs-contract coverage to enforce status for `5513`.

Out of scope:
- Runtime behavior changes.
- Broader historical spec normalization.

## Acceptance Criteria
- AC-1: `specs/5513/spec.md` reports `- Status: Implemented`.
- AC-2: Lifecycle docs-contract test enforces `5513` implemented status and non-regression from `Accepted`.
- AC-3: Targeted lifecycle tests pass.

## Conformance Cases
- C-01 (AC-1): `specs/5513/spec.md` contains `- Status: Implemented`.
- C-02 (AC-2): lifecycle docs-contract suite asserts implemented status for `5513` and rejects `Accepted` regression.
- C-03 (AC-3): targeted lifecycle test suite passes.

## Success Metrics / Observable Signals
- Closed merged telemetry task spec is lifecycle-consistent with implemented status contract.
- Docs-contract test prevents future status drift for `5513`.
