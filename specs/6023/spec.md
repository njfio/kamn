# Spec: Issue #6023 - Add M3 blind-index invariant unit tests

- Issue: #6023
- Status: Implemented
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26

## Problem Statement
`crates/kamn-core/src/data_layer_m3_blind_index_search.rs` has no direct tests despite implementing deterministic blind-index query behavior, duplicate registration guards, and blind-index determinism drift reporting.

## Scope
In scope:
- Add direct `#[cfg(test)]` coverage in `data_layer_m3_blind_index_search.rs`.
- Cover deterministic blind-index exact-match ordering behavior.
- Cover error-path rejection for invalid token and duplicate message-id registration.
- Cover determinism stable/drift decision projection.

Out of scope:
- Service API integration wiring.
- M4+ data-layer modules.

## Risk Level
`medium`

## Acceptance Criteria
- AC-1: Blind-index exact-match query behavior is directly validated with deterministic ordering.
- AC-2: Invalid token and duplicate message-id registration paths fail closed with deterministic errors.
- AC-3: Blind-index determinism evaluation emits stable/drift decisions with deterministic reason codes.

## Conformance Cases
- C-01 (Unit, AC-1): register scoped records and assert exact-match query returns deterministic message order.
- C-02 (Regression, AC-2): duplicate message-id registration and invalid token query return expected errors.
- C-03 (Unit, AC-3): determinism evaluation returns stable for matching baseline and drift for mismatched ordering baseline.

## Success Metrics / Observable Signals
- `cargo test -p kamn-core --lib data_layer_m3_blind_index_search -- --nocapture` passes with direct tests.
