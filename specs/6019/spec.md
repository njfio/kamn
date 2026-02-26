# Spec: Issue #6019 - Add M1 data-layer invariant unit tests

- Issue: #6019
- Status: Implemented
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26

## Problem Statement
`crates/kamn-core/src/data_layer_m1.rs` has no direct tests despite implementing deterministic merkle batch assembly, inclusion-proof verification/evaluation, and anchoring failure-matrix drift decisions.

## Scope
In scope:
- Add direct `#[cfg(test)]` coverage in `data_layer_m1.rs`.
- Cover deterministic batch/proof happy path.
- Cover invalid hash/proof rejection behavior.
- Cover anchoring failure-matrix stable/drift decision projection.

Out of scope:
- Cross-crate integration wiring.
- M2+ data-layer modules.

## Risk Level
`medium`

## Acceptance Criteria
- AC-1: M1 batch assembly + inclusion-proof verify/evaluate happy path is directly validated.
- AC-2: Invalid content hash and tampered proof paths fail closed with deterministic errors.
- AC-3: Anchoring failure-matrix stable/drift decisions are directly validated.

## Conformance Cases
- C-01 (Unit, AC-1): assembling unsorted leaves yields deterministic ordering and valid proof verification/evaluation.
- C-02 (Regression, AC-2): invalid content hash is rejected; tampered proof evaluates to invalid.
- C-03 (Unit, AC-3): failure matrix emits stable reason when observed values match expected and drift reason when mismatched.

## Success Metrics / Observable Signals
- `cargo test -p kamn-core data_layer_m1 -- --nocapture` passes with new tests.
