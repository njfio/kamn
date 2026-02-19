# Issue #5019 Plan

- Issue: #5019
- Status: Implemented

## Approach
1. Add red conformance tests for C-01..C-06 in `kamn-core`:
   - blind-index normalization determinism,
   - owner-salted index divergence,
   - exact-match blind-index lookup behavior,
   - metadata filter determinism and ordering,
   - fail-closed error handling for unsupported/empty search inputs.
2. Implement `data_layer_m3_blind_index_search` module with:
   - normalization + deterministic blind-index derivation,
   - owner-scoped exact-match blind-index lookup contracts,
   - metadata query filters with stable ordering.
3. Re-export M3 contracts from `crates/kamn-core/src/lib.rs`.
4. Execute format/lint/scoped/full regression and finalize lifecycle markers.

## Affected Modules
- `crates/kamn-core/src/data_layer_m3_blind_index_search.rs` (new)
- `crates/kamn-core/src/lib.rs` (module + re-exports)
- `crates/kamn-core/tests/data_layer_m3_blind_index_search.rs` (new)
- `specs/5019/spec.md`
- `specs/5019/plan.md`
- `specs/5019/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Risks:
  - Blind-index normalization drift causing missed matches or false mismatches.
  - Cross-owner leakage if owner scope is not enforced in lookup path.
  - Nondeterministic result ordering causing flaky query behavior.
- Mitigations:
  - Make normalization rules explicit and test equivalent-input classes first.
  - Encode owner scoping in both index registration and lookup predicates.
  - Use deterministic sort keys (`created_at`, then `message_id`) and verify ordering in tests.
  - Keep implementation Rust-only to preserve shell ratio constraints.

## Interface Contract
- Additive public API under `kamn_core::data_layer_m3_blind_index_search::*`.
- No dependency additions.
- No protocol/wire-format changes.

## ADR
- Not required for this scoped additive implementation.
