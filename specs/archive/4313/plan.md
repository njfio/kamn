# Issue #4313 Plan

- Issue: `#4313`
- Status: `Completed`

## Approach
- Integrate peer integrity drift and retry-timeout conformance tests to lock deterministic behavior.
- Add deterministic peer adapter reason projection and multi-process validation hook surfaces in `p2p_transport`.
- Export additive peer adapter APIs through `kamn-core::lib`.
- Add peer transport integrity/reason governance marker sections in planning/release docs and enforce parity through docs tests.

## Affected Modules
- `crates/kamn-core/src/p2p_transport.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/p2p_peer_integrity_drift_timeout.rs`
- `crates/kamn-core/tests/p2p_peer_adapter_reason_projection.rs`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `docs/planning/kolme-devnet-ops.md`
- `docs/foundation/release-gonogo-checklist.md`

## Risks and Mitigations
- Risk: reconnect reason mapping can drift between timeout and budget-exhausted classes.
- Mitigation: deterministic projection APIs and regression tests around attempt boundaries.
- Risk: multi-process validation hooks can become unordered/non-deterministic over time.
- Mitigation: hook ordering/taxonomy tests and docs markers guarded in fast lanes.

## Interface Contract
- Additive `kamn-core` exports for peer adapter reason projection and deterministic multi-process validation hooks.

## ADR
- Not required.
