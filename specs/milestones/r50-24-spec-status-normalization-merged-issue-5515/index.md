# Milestone R50.24 - Spec Status Normalization for Merged Issue 5515

## Objective
Normalize lifecycle closure metadata for merged issue `#5515` and extend guardrail coverage so merged status-normalization tasks cannot drift to `Accepted`.

## Scope
- Create and execute issue `#5517` to update `specs/5515/spec.md` to `- Status: Implemented`.
- Extend `review_r50_spec_status_lifecycle_docs_contract` coverage to assert `5515` status semantics.
- Validate via targeted docs-contract and regression lanes.

## Deliverables
- `specs/5517/spec.md`
- `specs/5517/plan.md`
- `specs/5517/tasks.md`
- `crates/kamn-core/tests/review_r50_spec_status_lifecycle_docs_contract.rs` updates
- `specs/5515/spec.md` status normalization

## Exit Criteria
- `#5517` merged with all ACs satisfied.
- `specs/5515/spec.md` is lifecycle-consistent with `Implemented`.
- R50 lifecycle guardrail includes explicit regression protection for `5515`.
