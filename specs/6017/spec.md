# Spec: Issue #6017 - Add M0 data-layer invariant unit tests

- Issue: #6017
- Status: Implemented
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26

## Problem Statement
`crates/kamn-core/src/data_layer_m0.rs` has no direct tests despite being foundational for append-only ledger integrity, deterministic record derivation, and conformance decision reporting.

## Scope
In scope:
- Add direct `#[cfg(test)]` coverage in `data_layer_m0.rs`.
- Cover append/hash-chain happy path and tamper/duplicate failure paths.
- Cover conformance matrix decision stability/drift behavior.

Out of scope:
- Cross-crate integration harness for M1+ modules.
- Storage backend wiring changes.

## Risk Level
`medium`

## Acceptance Criteria
- AC-1: M0 append/hash-chain happy path is verified by direct unit tests.
- AC-2: Duplicate-message and tamper-detection error paths are covered.
- AC-3: Conformance matrix stable/drift decisions are covered.

## Conformance Cases
- C-01 (Unit, AC-1): appending two records produces expected chain links and `verify_hash_chain` succeeds.
- C-02 (Regression, AC-2): duplicate append returns `DuplicateMessageId`; tampered content hash returns `InvalidHashChainLink`.
- C-03 (Unit, AC-3): conformance matrix emits stable reason when all cases match and drift reason when one mismatches.

## Success Metrics / Observable Signals
- `cargo test -p kamn-core data_layer_m0 -- --nocapture` passes with new tests.
