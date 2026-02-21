# Issue #5511 Spec - R50 Spec Status Lifecycle Normalization

- Status: Accepted
- Issue: #5511
- Parent: #5469
- Milestone: R50.21 Spec status lifecycle normalization for completed tasks

## Problem Statement
Merged/closed R50 task specs (`5507`, `5509`) still report `Status: Accepted`, violating the closure contract requiring `Status: Implemented`.

## Scope
In scope:
- Add docs-contract enforcement for implemented status on targeted closed R50 task specs.
- Update `specs/5507/spec.md` and `specs/5509/spec.md` to `Status: Implemented`.

Out of scope:
- Runtime/code behavior changes.
- Global historical spec-status normalization outside targeted R50 tasks.

## Acceptance Criteria
- AC-1: `specs/5507/spec.md` status is `Implemented`.
- AC-2: `specs/5509/spec.md` status is `Implemented`.
- AC-3: Dedicated docs-contract test enforces implemented status for targeted specs.
- AC-4: Targeted tests pass.

## Conformance Cases
- C-01 (AC-1): `specs/5507/spec.md` contains `- Status: Implemented`.
- C-02 (AC-2): `specs/5509/spec.md` contains `- Status: Implemented`.
- C-03 (AC-3): docs-contract test fails on non-implemented state and passes on implemented state.
- C-04 (AC-4): `cargo test -p kamn-core --test review_r50_spec_status_lifecycle_docs_contract` passes.

## Success Metrics / Observable Signals
- Closed R50 task specs in scope are lifecycle-consistent with contract status.
- CI-enforced test prevents regression to non-implemented status.
