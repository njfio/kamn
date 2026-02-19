# Issue #5020 Plan

- Issue: #5020
- Status: Implemented

## Approach
1. Add red conformance tests for C-01..C-06 in `kamn-core`:
   - escrow lifecycle transition matrix,
   - invalid transition fail-closed behavior,
   - escrow visibility matrix with auditor threshold gate,
   - settlement evidence append+verify and tamper detection.
2. Implement `data_layer_m4_escrow_integration` module with:
   - deterministic escrow state machine contract,
   - escrow-scoped visibility evaluator,
   - append-only settlement evidence registry with integrity verification.
3. Re-export M4 contracts from `crates/kamn-core/src/lib.rs`.
4. Execute format/lint/scoped/full regression and finalize lifecycle markers.

## Affected Modules
- `crates/kamn-core/src/data_layer_m4_escrow_integration.rs` (new)
- `crates/kamn-core/src/lib.rs` (module + re-exports)
- `crates/kamn-core/tests/data_layer_m4_escrow_integration.rs` (new)
- `specs/5020/spec.md`
- `specs/5020/plan.md`
- `specs/5020/tasks.md`

## Risks and Mitigations
- Risk level: high
- Risks:
  - Transition drift between escrow lifecycle state and settlement evidence acceptance rules.
  - Visibility leaks if auditor gating does not enforce dispute+threshold requirements.
  - Integrity checks that fail to detect evidence tampering.
- Mitigations:
  - Encode transition map explicitly and test invalid edges first.
  - Require dispute-active + share-threshold satisfaction for auditor allow path.
  - Bind evidence records to deterministic digests and verify full registry integrity.
  - Keep implementation Rust-only to preserve shell ratio constraints.

## Interface Contract
- Additive public API under `kamn_core::data_layer_m4_escrow_integration::*`.
- No dependency additions.
- No protocol/wire-format changes.

## ADR
- Not required for this scoped additive implementation.

## Execution Outcome
- Added `data_layer_m4_escrow_integration` module implementing escrow lifecycle transitions,
  visibility authorization, and settlement evidence hash-chain verification.
- Added and passed conformance tests `spec_c01`..`spec_c05` in `kamn-core`.
- Kept implementation Rust-only; no shell/python/workflow/template changes.
