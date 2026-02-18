# Issue #5024 Plan

- Issue: #5024
- Status: Implemented

## Approach
1. Add red conformance tests (`spec_c01`..`spec_c05`) for:
   - CEK tombstoning and integrity-field preservation on shred,
   - retention-window eligibility across M8 classes and extension windows,
   - legal-hold precedence on retention/shredding,
   - owner-scope fail-closed boundaries.
2. Implement `data_layer_m8_compliance_lifecycle` module with:
   - owner-scoped message retention registry,
   - deterministic retention evaluator and due-candidate projector,
   - legal-hold apply/release controls,
   - crypto-shred transition with CEK tombstone markers.
3. Re-export M8 APIs in `crates/kamn-core/src/lib.rs`.
4. Run format/lint/targeted/full regression and finalize lifecycle evidence.

## Affected Modules
- `crates/kamn-core/src/data_layer_m8_compliance_lifecycle.rs` (new)
- `crates/kamn-core/src/lib.rs` (module + exports)
- `crates/kamn-core/tests/data_layer_m8_compliance_lifecycle.rs` (new)
- `specs/5024/spec.md`
- `specs/5024/plan.md`
- `specs/5024/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigations:
  - Enforce explicit owner-scope authorization gates at all query/mutation entry points.
  - Keep deterministic sorting and reason markers for retention outputs.
  - Preserve append-only integrity markers across shredding transitions.
  - Keep implementation Rust-only to maintain shell-ratio constraints.

## Interface Contract
- Additive public API only under `kamn_core::data_layer_m8_compliance_lifecycle::*`.
- No new dependencies.
- No protocol/wire-format changes.

## ADR
- Not required for this scoped additive implementation.

## Outcome
- Added `data_layer_m8_compliance_lifecycle` for owner-scoped retention due evaluation, legal-hold gating, and crypto-shredding transitions.
- Re-exported the full M8 contract API through `kamn_core`.
- Landed and passed conformance suite `spec_c01`..`spec_c05` and full `kamn-core` regression.
