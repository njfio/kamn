# Issue #3926 Plan

- Issue: `#3926`
- Status: `Completed`

## Approach
- Add one regression matrix test in `runtime_tests.rs` that explicitly validates deterministic action-to-reason mapping for Accept, SlowProducer (Suspend alias), Reject, and Purge.
- Extend live transport regression tests to verify dispatch remains successful through SlowProducer range and fails closed at reject threshold.
- Update runtime-network docs with explicit decision marker matrix and add docs-contract assertions for those markers.

## Affected Modules
- `crates/kamn-core/src/runtime_tests.rs`
- `crates/kamn-core/tests/p2p_live_transport_runtime.rs`
- `crates/kamn-core/tests/runtime_network_docs.rs`
- `docs/foundation/runtime-network.md`

## Risks and Mitigations
- Risk: test assumptions about threshold boundaries become brittle.
- Mitigation: assert behavior by ranges and deterministic reason markers rather than hardcoded queue internals only.
- Risk: docs-marker checks over-constrain prose changes.
- Mitigation: assert compact marker lines dedicated to taxonomy contracts.

## Interfaces and Contracts
- No public API changes.
- Regression and docs contracts only; behavior remains fail-closed.

## ADR
- No ADR required: no dependency or architecture change.
