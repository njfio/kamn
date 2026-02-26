# Spec: Issue #6027 - Add core invariants unit tests for data_layer_m5

- Issue: #6027
- Status: Implemented
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #5976

## Problem Statement
`crates/kamn-core/src/data_layer_m5_vector_integration.rs` has no direct module unit tests despite implementing deterministic embedding append/hash-chain integrity, semantic query ranking, recall-drift decisions, and anomaly classification contracts.

## Scope
In scope:
- Add direct `#[cfg(test)]` coverage in `data_layer_m5_vector_integration.rs`.
- Validate deterministic append and owner integrity verification paths.
- Validate fail-closed paths (privacy violations / tampered hash chain / dimension mismatch).
- Validate recall drift stable/degraded decision projection reason codes.

Out of scope:
- Cross-crate integration wiring.
- Service API runtime behavior changes.
- M6+ module coverage.

## Risk Level
`medium`

## Acceptance Criteria
- AC-1: Append flow records deterministic owner-chain metadata and passes owner integrity verification.
- AC-2: Tampered hash chain and privacy/dimension violations fail closed with deterministic error taxonomy.
- AC-3: Recall drift evaluation deterministically projects `Stable` and `Degraded` decisions with stable reason codes.

## Conformance Cases
- C-01 (Unit, AC-1): Server-side plaintext registry append creates deterministic sequence/hash-chain and `verify_owner_integrity` passes.
- C-02 (Regression, AC-2): Tampering one record hash causes `InvalidEmbeddingHashChain` fail-closed reason.
- C-03 (Functional, AC-2): Owner-side encrypted mode rejects plaintext append and semantic query as unavailable.
- C-04 (Conformance, AC-3): Recall drift reports stable for baseline-preserving query and degraded when baseline is missing/shifted beyond threshold.

## Success Metrics / Observable Signals
- New direct M5 tests execute in `kamn-core` suite and pass consistently.
- AC-to-test mapping is explicit in PR verification table.
- M5 no longer appears as zero-direct-unit-coverage in gap tracking.
