# Issue #5036 Plan

- Issue: #5036
- Status: Implemented

## Approach
1. Add RED conformance tests for M7 billing reconciliation match/mismatch,
   owner-scope denial, and invalid bucket alignment.
2. Implement additive reconciliation API in M7 module with deterministic reason
   markers and explicit projected/statement total echoing.
3. Keep existing M7 aggregate and billing projection behavior unchanged.
4. Run scoped/full regression and shell guardrail evidence commands.

## Affected Modules
- `crates/kamn-core/src/data_layer_m7_timeseries_telemetry.rs`
- `crates/kamn-core/tests/data_layer_m7_timeseries_telemetry.rs`
- `crates/kamn-core/src/lib.rs`
- `specs/5036/spec.md`
- `specs/5036/plan.md`
- `specs/5036/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep reconciliation logic additive and deterministic.
  - Reuse existing owner-scope authorization path for fail-closed behavior.
  - Keep implementation Rust-only; no shell/workflow changes.

## Interface Contract
- Additive API in `kamn_core::data_layer_m7_timeseries_telemetry`.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this scoped additive contract.
