# Issue #5038 Plan

- Issue: #5038
- Status: Implemented

## Approach
1. Add RED conformance tests for queue-ordering visibility and duplicate-id
   rejection markers in `data_layer_m9_realtime_delivery`.
2. Implement additive queue snapshot API exposing deterministic
   pending/deferred order under owner-scope authorization.
3. Re-run/adjust presence and backpressure tests to keep deterministic reason
   marker guarantees.
4. Run scoped/full regression and shell guardrail evidence commands.

## Affected Modules
- `crates/kamn-core/src/data_layer_m9_realtime_delivery.rs`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs`
- `specs/5038/spec.md`
- `specs/5038/plan.md`
- `specs/5038/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep additive API surface minimal and deterministic.
  - Preserve fail-closed owner-scope checks on new queue snapshot API.
  - Keep implementation Rust-only; avoid shell/workflow changes.

## Interface Contract
- Additive API in `kamn_core::data_layer_m9_realtime_delivery` only.
- No dependency/protocol/wire-format change.

## ADR
- Not required for this scoped additive contract update.
