# Issue #5041 Plan

- Issue: #5041
- Status: Implemented

## Approach
1. Add RED conformance tests (`spec_c01`..`spec_c05`) for:
   - verified outcome under shell-neutral + ratio-compliant evidence,
   - blocked outcomes for shell-mode violations, shell growth, and ratio fail,
   - warning outcome for ratio warn window,
   - fail-closed threshold validation.
2. Implement `data_layer_shell_neutral_policy` module with:
   - input/evaluation/report models,
   - decision enum (`Verified`/`Warning`/`Blocked`),
   - deterministic reason-marker ordering.
3. Re-export policy APIs from `crates/kamn-core/src/lib.rs`.
4. Run format/lint/targeted/full regression and shell guardrail evidence.

## Affected Modules
- `crates/kamn-core/src/data_layer_shell_neutral_policy.rs` (new)
- `crates/kamn-core/src/lib.rs` (module + exports)
- `crates/kamn-core/tests/data_layer_shell_neutral_policy.rs` (new)
- `specs/5041/spec.md`
- `specs/5041/plan.md`
- `specs/5041/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Reuse `DataLayerPrdCriticalScenarioConformanceReport` to avoid policy drift.
  - Keep threshold validation fail-closed and deterministic.
  - Keep implementation Rust-only to avoid shell-surface growth.

## Interface Contract
- Additive public API under `kamn_core::data_layer_shell_neutral_policy::*`.
- No new dependencies.
- No protocol/wire-format changes.

## ADR
- Not required for this scoped additive implementation.
