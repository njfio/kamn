# Issue #5517 Spec - Spec Status Normalization for Merged Issue 5515

- Status: Implemented
- Issue: #5517
- Parent: #5515
- Milestone: R50.24 Spec status normalization for merged issue 5515

## Problem Statement
Issue `#5515` is merged/closed but `specs/5515/spec.md` still reports `Status: Accepted`, violating lifecycle closure contract expectations.

## Scope
In scope:
- Update `specs/5515/spec.md` status line to `Implemented`.
- Extend lifecycle docs-contract coverage to enforce status for `5515`.

Out of scope:
- Runtime behavior changes.
- Broad historical spec normalization.

## Acceptance Criteria
- AC-1: `specs/5515/spec.md` reports `- Status: Implemented`.
- AC-2: Lifecycle docs-contract test enforces `5515` implemented status and non-regression from `Accepted`.
- AC-3: Targeted lifecycle tests pass.

## Conformance Cases
- C-01 (AC-1): `specs/5515/spec.md` contains `- Status: Implemented`.
- C-02 (AC-2): lifecycle docs-contract suite asserts implemented status for `5515` and rejects `Accepted` regression.
- C-03 (AC-3): targeted lifecycle test suite passes.

## Success Metrics / Observable Signals
- Closed merged status-normalization task spec is lifecycle-consistent with implemented status contract.
- Docs-contract test prevents future status drift for `5515`.
