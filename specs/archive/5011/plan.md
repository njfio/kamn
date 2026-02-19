# Issue #5011 Plan

- Issue: #5011
- Status: Implemented

## Approach
1. Deliver M8 compliance contracts through child task `#5024`:
   - deterministic retention due evaluation,
   - legal-hold precedence controls,
   - crypto-shred transitions preserving integrity markers.
2. Preserve additive exports in `kamn-core` for downstream integration lanes.
3. Validate with scoped suite `data_layer_m8_compliance_lifecycle` and crate-level regression.
4. Keep delivery Rust-only for this story to preserve shell budget neutrality.

## Affected Modules
- `crates/kamn-core/src/data_layer_m8_compliance_lifecycle.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m8_compliance_lifecycle.rs`
- `specs/5011/spec.md`
- `specs/5011/plan.md`
- `specs/5011/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigations:
  - Keep retention and legal-hold transitions deterministic in conformance coverage.
  - Preserve strict owner-scope enforcement on all mutation/query paths.
  - Preserve rust-only implementation to avoid shell-surface growth.

## Interface Contract
- Additive API/exports in `kamn-core`.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this bounded additive story closure.
