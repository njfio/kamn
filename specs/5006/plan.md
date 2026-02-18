# Issue #5006 Plan

- Issue: #5006
- Status: Implemented

## Approach
1. Deliver M3 search contracts through child task `#5019`:
   - deterministic blind-index normalization and owner-scoped token derivation,
   - exact-match blind-index search behavior with fail-closed invalid-mode handling,
   - deterministic metadata filtering and ordering contracts.
2. Preserve additive exports in `kamn-core` for downstream M4+ integration.
3. Validate with scoped suite `data_layer_m3_blind_index_search` and crate-level regression.
4. Keep delivery Rust-only for this story to preserve shell budget neutrality.

## Affected Modules
- `crates/kamn-core/src/data_layer_m3_blind_index_search.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m3_blind_index_search.rs`
- `specs/5006/spec.md`
- `specs/5006/plan.md`
- `specs/5006/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep normalization and owner-scope rules explicit in conformance tests.
  - Preserve deterministic ordering to avoid flaky query behavior.
  - Preserve rust-only implementation to avoid shell-surface growth.

## Interface Contract
- Additive API/exports in `kamn-core`.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this bounded additive story closure.
