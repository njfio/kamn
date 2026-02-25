# Spec: Issue #5922 - Task: Replace fake SHA-256 labels with real sha2::Sha256 in data layer M0-M5

- Issue: #5922
- Status: Implemented
- Type: task
- Priority: P0
- Area: security
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5916

## Problem Statement
Current functions label outputs as sha256 while using custom FNV-like mixing.

## Scope
In scope:
- Introduce shared digest utility backed by sha2::Sha256; migrate M0-M5 call sites.

Out of scope:
- Data-model redesign unrelated to hash correctness.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: All M0-M5 digest outputs are computed with real SHA-256.
- AC-2: Duplicate deterministic_digest_256_hex implementations are removed.
- AC-3: Compatibility/regression tests verify expected digest format and deterministic outputs for fixed vectors.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): `regression_issue_5922_m3_blind_index_digest_matches_real_sha256_vector` verifies M3 blind-index digest output matches canonical SHA-256 vector.
- C-02 (Regression, AC-2): `regression_issue_5922_m0_m5_remove_custom_digest_mixers` verifies M0-M5 removed `deterministic_digest_256_hex` and route through shared `tagged_sha256`.
- C-03 (Unit/Functional, AC-3): `data_layer_hashing::tests::regression_issue_5922_sha256_hex_matches_known_test_vectors` verifies fixed SHA-256 vectors and compatibility output format.
- C-04 (Integration, AC-4): `data_layer_m0_contract`, `data_layer_m1_anchoring_orchestrator`, `data_layer_m2_gateway_access`, `data_layer_m3_blind_index_search`, `data_layer_m4_escrow_integration`, and `data_layer_m5_vector_integration` suites pass.
- C-05 (Verify, AC-4): `cargo fmt --check` and strict `kamn-core` clippy pass for touched modules.

## Success Metrics / Observable Signals
- All M0-M5 tagged digests are computed via shared SHA-256 helper (`data_layer_hashing::tagged_sha256`).
- Legacy pseudo-digest helper duplication is removed across M0-M5 modules.
- SHA-256 fixed-vector tests and data-layer integration suites pass.
- Scoped formatting and strict clippy checks pass.


## Required Test Categories
- Unit: digest utility vectors
- Functional: M0-M5 module hash outputs
- Integration: append-only ledger integrity path
- Regression: old fake hash path absent
- Performance: hash throughput non-regression check

## Dependencies
- #5916
