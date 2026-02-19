# Issue #5021 Plan

- Issue: #5021
- Status: Implemented

## Approach
1. Add red conformance tests for C-01..C-06 in `kamn-core`:
   - append-only deterministic embedding registry,
   - duplicate-id fail-closed behavior,
   - owner-scoped semantic query deterministic ranking,
   - privacy-mode gating denials,
   - centroid-distance anomaly threshold classification.
2. Implement `data_layer_m5_vector_integration` module with:
   - deterministic embedding record ingestion + hash chain,
   - semantic query service (owner-scoped, cosine-similarity top-k),
   - anomaly evaluator (agent centroid + threshold reason markers).
3. Re-export M5 contracts from `crates/kamn-core/src/lib.rs`.
4. Execute format/lint/scoped/full regression and finalize lifecycle markers.

## Affected Modules
- `crates/kamn-core/src/data_layer_m5_vector_integration.rs` (new)
- `crates/kamn-core/src/lib.rs` (module + re-exports)
- `crates/kamn-core/tests/data_layer_m5_vector_integration.rs` (new)
- `specs/5021/spec.md`
- `specs/5021/plan.md`
- `specs/5021/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigations:
  - Keep all query ordering deterministic via explicit sorting/tie-break rules.
  - Fail closed when vectors are missing, malformed, cross-owner, or dimension-mismatched.
  - Validate privacy-mode preconditions before semantic query evaluation.
  - Keep implementation Rust-only to preserve shell ratio constraints.

## Interface Contract
- Additive public API under `kamn_core::data_layer_m5_vector_integration::*`.
- No dependency additions.
- No protocol/wire-format changes.

## ADR
- Not required for this scoped additive implementation.

## Execution Outcome
- Added `data_layer_m5_vector_integration` module implementing deterministic
  embedding append/hash-chain contracts, owner-scoped semantic query ranking,
  and centroid-distance anomaly evaluation.
- Added and passed conformance tests `spec_c01`..`spec_c05` in `kamn-core`.
- Kept implementation Rust-only; no shell/python/workflow/template changes.
