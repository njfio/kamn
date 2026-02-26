# Spec: Issue #6029 - Add core invariants unit tests for data_layer_m9_gateway_bridge

- Issue: #6029
- Status: Implemented
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #5976

## Problem Statement
`crates/kamn-core/src/data_layer_m9_gateway_bridge.rs` has no direct module unit tests despite projecting deterministic transport/dispatch/presence envelopes used at the M9 gateway boundary.

## Scope
In scope:
- Add direct `#[cfg(test)]` coverage in `data_layer_m9_gateway_bridge.rs`.
- Validate unsupported transport profiles fail closed with stable reason codes.
- Validate dispatch projection emits deterministic envelope fields from M9 dispatch outcomes.
- Validate presence projection emits deterministic visible/not-found envelopes and audit tags.

Out of scope:
- Runtime delivery wiring across node/network services.
- Transport protocol redesign.
- Changes to M9 realtime domain APIs.

## Risk Level
`medium`

## Acceptance Criteria
- AC-1: Unsupported transport profile inputs fail closed with `UnsupportedTransport` and stable reason marker.
- AC-2: Dispatch projection normalizes transport profile labels and maps M9 dispatch outcome fields without mutation.
- AC-3: Presence projection emits deterministic visible/not-found envelopes including stable reason codes and deterministic audit tags.

## Conformance Cases
- C-01 (Unit, AC-1): Unsupported transport label returns `DataLayerM9GatewayBridgeError::UnsupportedTransport` with `m9_gateway_unsupported_transport`.
- C-02 (Functional, AC-2): Gateway dispatch projection returns `m9.dispatch.ack` envelope with normalized `websocket` transport and deterministic ack metadata.
- C-03 (Conformance, AC-3): Presence projection returns `visible=true` when target is connected and `visible=false` with not-found reason for a disconnected target while retaining deterministic `audit_record_tag` prefix.

## Success Metrics / Observable Signals
- New direct M9 gateway bridge tests pass in `kamn-core`.
- AC-to-test mapping is explicit in PR verification table.
- `data_layer_m9_gateway_bridge` no longer appears in zero-direct-unit-coverage tracking.
