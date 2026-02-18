# Issue #5025 Plan

- Issue: #5025
- Status: Implemented

## Approach
1. Add RED conformance tests (`spec_c01`..`spec_c05`) for:
   - connected-recipient delivered ACK behavior,
   - scoped presence visibility checks,
   - deterministic queue-cap backpressure escalation markers,
   - owner-scope fail-closed controls.
2. Implement `data_layer_m9_realtime_delivery` module with:
   - owner-scoped presence registry + linkage gates,
   - dispatch ACK decision engine (Delivered vs Queued),
   - per-recipient queue cap and sustained-pressure escalation flags.
3. Re-export M9 API in `crates/kamn-core/src/lib.rs`.
4. Run format/lint/targeted/full regression and finalize evidence markers.

## Affected Modules
- `crates/kamn-core/src/data_layer_m9_realtime_delivery.rs` (new)
- `crates/kamn-core/src/lib.rs` (module + exports)
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs` (new)
- `specs/5025/spec.md`
- `specs/5025/plan.md`
- `specs/5025/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigations:
  - Keep queue/backpressure policies deterministic with explicit constants.
  - Enforce owner-scope authorization at every mutation/query entrypoint.
  - Use stable reason markers and sorted outputs for reproducible checks.
  - Keep implementation Rust-only to avoid shell-surface growth.

## Interface Contract
- Additive public API under `kamn_core::data_layer_m9_realtime_delivery::*`.
- No new dependencies.
- No protocol/wire-format changes.

## ADR
- Not required for this scoped additive implementation.

## Outcome
- Added `data_layer_m9_realtime_delivery` contracts for owner-scoped dispatch ACK behavior, presence visibility gating, and queue-cap backpressure escalation markers.
- Re-exported full M9 API through `kamn_core`.
- Landed and passed conformance suite `spec_c01`..`spec_c05` plus full `kamn-core` regression.
