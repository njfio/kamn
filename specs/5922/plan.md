# Plan: Issue #5922 - Task: Replace fake SHA-256 labels with real sha2::Sha256 in data layer M0-M5

- Issue: #5922
- Spec: `specs/5922/spec.md`
- Status: Implemented
- Last Updated: 2026-02-24

## Approach
1. RED: added failing contract tests for M3 fixed-vector SHA-256 output and M0-M5 pseudo-digest helper removal.
2. Implemented shared digest utility in `crates/kamn-core/src/data_layer_hashing.rs` using `k256::sha2::Sha256`.
3. Migrated M0-M5 tagged digest call sites to shared `tagged_sha256` and removed local `deterministic_digest_256_hex` implementations.
4. Verified with issue contract tests, M0-M5 integration suites, `cargo fmt --check`, and strict `kamn-core` clippy.

## Affected Modules
- `crates/kamn-core/src/data_layer_hashing.rs`
- `crates/kamn-core/src/data_layer_m0.rs`
- `crates/kamn-core/src/data_layer_m1.rs`
- `crates/kamn-core/src/data_layer_m2_gateway_access.rs`
- `crates/kamn-core/src/data_layer_m3_blind_index_search.rs`
- `crates/kamn-core/src/data_layer_m4_escrow_integration.rs`
- `crates/kamn-core/src/data_layer_m5_vector_integration.rs`
- `crates/kamn-core/tests/data_layer_sha256_contract.rs`

## Risks + Mitigations
- Risk: Scope expansion across multiple crates can increase merge and verification time.
  - Mitigation: keep PRs task-scoped and verify each AC with targeted tests before running broader suites.
- Risk: Security/runtime changes can regress existing contracts.
  - Mitigation: preserve and extend regression coverage before behavior changes.
- Risk: Cross-issue dependencies can block downstream tasks.
  - Mitigation: execute in dependency order and keep blockers logged in issue comments.

## Interfaces / Contracts
- Primary contract source: `specs/5922/spec.md`.
- Upstream issue contract: GitHub issue #5922.
- Protocol/API/schema changes require explicit documentation updates and linked follow-up issues when out of scope.

## ADR Requirement
- Not required (no dependency or protocol/wire-format change).
