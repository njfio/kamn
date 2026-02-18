# Issue #5016 Plan

- Issue: #5016
- Status: Reviewed

## Approach
1. Add a new Rust module `data_layer_m0` with:
   - deterministic record derivation from `CanonicalMessageEnvelope` + ciphertext metadata,
   - append-only in-memory ledger,
   - hash-chain verification entry point.
2. Write conformance tests first (`spec_c01`..`spec_c04`) in red state.
3. Implement minimal code to satisfy the tests, including explicit error taxonomy.
4. Export the module from `lib.rs` for follow-on milestones.
5. Run scoped tests (`cargo test -p kamn-core data_layer_m0`) and crate regression (`cargo test -p kamn-core`).

## Affected Modules
- `crates/kamn-core/src/data_layer_m0.rs` (new)
- `crates/kamn-core/src/lib.rs` (module + re-exports)
- `specs/5016/{spec.md,plan.md,tasks.md}` (lifecycle updates)

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep implementation intentionally in-memory and deterministic; DB wiring deferred to later issues.
  - Use canonical serialization with sorted collections to avoid nondeterministic hash drift.
  - No dependency additions in this task to respect AGENTS dependency approval rule.

## Interface Contract
- No protocol/wire-format changes.
- New public API is additive under `kamn_core::data_layer_m0::*`.
- Hash primitive is deterministic foundation-level and explicitly documented for later swap to SHA-256 backend.

## ADR
- Not required for this scoped additive module (no dependency or architecture pivot).
