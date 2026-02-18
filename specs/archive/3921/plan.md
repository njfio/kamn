# Issue #3921 Plan

- Issue: #3921
- Status: Implemented

## Approach (Implemented)
1. Land live transport enqueue enforcement using deterministic runtime backpressure decisions.
2. Propagate deterministic reason codes into transport error/runtime event projections.
3. Add regression tests for decision matrix and dispatch-path behavior.
4. Update runtime-network docs to codify backpressure decision markers.

## Delivered Through
- PR #4998
- PR #4999

## Affected Modules
- `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs`
- `crates/kamn-core/src/p2p_transport/error.rs`
- `crates/kamn-core/src/p2p_transport/runtime_event.rs`
- `crates/kamn-core/src/runtime_tests.rs`
- `crates/kamn-core/tests/p2p_live_transport_runtime.rs`
- `crates/kamn-core/tests/runtime_network_docs.rs`
- `docs/foundation/runtime-network.md`

## Risks and Mitigations
- Risk level: medium
- Mitigation:
  - Deterministic reason-code taxonomy covered by explicit regression tests.
  - Dispatch behavior verified at threshold boundaries in integration tests.

## Interface Contract
- No dependency or wire-format changes.
- Enforcement remains internal to runtime dispatch + queue behavior projection.

## ADR
- No ADR required (no architecture boundary change beyond existing runtime design).
