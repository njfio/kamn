# Spec: Issue #6025 - Add M4 escrow-integration invariant unit tests

- Issue: #6025
- Status: Implemented
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26

## Problem Statement
`crates/kamn-core/src/data_layer_m4_escrow_integration.rs` currently has no direct tests despite implementing escrow transition contracts, visibility authorization decisions, evidence hash-chain integrity, and reconciliation decisions.

## Scope
In scope:
- Add direct `#[cfg(test)]` coverage in `data_layer_m4_escrow_integration.rs`.
- Cover deterministic transition + visibility happy-path behavior.
- Cover invalid transition and tampered evidence hash-chain rejection paths.
- Cover reconciliation match/mismatch decision projection.

Out of scope:
- Service API integration wiring.
- M5+ data-layer modules.

## Risk Level
`medium`

## Acceptance Criteria
- AC-1: Escrow transition and participant visibility happy paths are directly validated with deterministic reason codes.
- AC-2: Invalid transition and evidence tamper paths fail closed with deterministic typed errors.
- AC-3: Reconciliation emits match/mismatch decisions with deterministic reason codes.

## Conformance Cases
- C-01 (Unit, AC-1): create escrow, apply valid transitions, and verify participant visibility allow + outsider deny reason codes.
- C-02 (Regression, AC-2): invalid transition from `Created` via `Activate` and tampered evidence hash-chain both return deterministic errors.
- C-03 (Unit, AC-3): reconcile terminal escrow against matching evidence (match) and diverging latest evidence (mismatch).

## Success Metrics / Observable Signals
- `cargo test -p kamn-core --lib data_layer_m4_escrow_integration -- --nocapture` passes with direct tests.
