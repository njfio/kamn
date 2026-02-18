# Issue #5012 Plan

- Issue: #5012
- Status: Implemented

## Approach
1. Deliver M9 realtime contracts through child task `#5025`:
   - deterministic dispatch ACK outcomes,
   - fail-closed scoped presence visibility,
   - deterministic queue-cap backpressure escalation markers.
2. Preserve additive exports in `kamn-core` for downstream integration lanes.
3. Validate with scoped suite `data_layer_m9_realtime_delivery` and crate-level regression.
4. Keep delivery Rust-only for this story to preserve shell budget neutrality.

## Affected Modules
- `crates/kamn-core/src/data_layer_m9_realtime_delivery.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs`
- `specs/5012/spec.md`
- `specs/5012/plan.md`
- `specs/5012/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep ACK and escalation thresholds deterministic in conformance coverage.
  - Preserve strict owner-scope enforcement across dispatch/presence paths.
  - Preserve rust-only implementation to avoid shell-surface growth.

## Interface Contract
- Additive API/exports in `kamn-core`.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this bounded additive story closure.
