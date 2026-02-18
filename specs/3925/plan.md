# Issue #3925 Plan

- Issue: `#3925`
- Status: `Completed`

## Approach
- Add a shared live-transport inbox enqueue helper in `p2p_transport_live.rs` that evaluates deterministic runtime backpressure policy before mutating queue state.
- Use the helper from both dispatch surfaces:
  - contract data-plane `send()` path (`cfg(not(feature = "libp2p-live-transport"))`)
  - native swarm receive path (`apply_libp2p_swarm_event_to_live_state` under feature gate)
- Extend p2p transport error/runtime event taxonomies for backpressure reject/purge reason-code projection.
- Add focused regression/integration tests covering saturated reject behavior and reason-code stability.

## Affected Modules
- `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs`
- `crates/kamn-core/src/p2p_transport/error.rs`
- `crates/kamn-core/src/p2p_transport/runtime_event.rs`
- `crates/kamn-core/tests/p2p_live_transport_runtime.rs`

## Risks and Mitigations
- Risk: queue thresholds could break existing live transport tests.
- Mitigation: set conservative queue capacity and validate full `p2p_live_transport_runtime` suite.
- Risk: feature and non-feature dispatch paths diverge.
- Mitigation: route both through one shared helper and add coverage for both reason-code surfaces.
- Risk: runtime event taxonomy drift.
- Mitigation: add regression assertions for deterministic reason codes.

## Interfaces and Contracts
- `PeerLifecycleTransport::send` keeps signature and fail-closed semantics; may return new deterministic backpressure variants.
- `P2pTransportError::reason_code()` extends deterministic mapping with runtime backpressure markers.
- `Libp2pBehaviorFailureClass` extends deterministic behavior-failure markers for backpressure reject/purge.

## ADR
- No ADR required: no dependency, wire protocol, or architectural boundary change.
