# Issue #5028 Plan

- Issue: #5028
- Status: Implemented

## Approach
1. Add RED conformance tests (`spec_c01`..`spec_c05`) for:
   - deterministic required-scenario catalog (`62..71`),
   - fully passing matrix -> `Conformant`,
   - scenario failure/shell-policy violation -> `NonConformant`,
   - invalid scenario id + record mutation fail-closed errors.
2. Implement `data_layer_prd_critical_scenario_conformance` module with:
   - required scenario catalog and result registry,
   - deterministic conformance evaluator with stable reason markers,
   - shell-neutral orchestration mode policy enforcement.
3. Re-export APIs in `crates/kamn-core/src/lib.rs`.
4. Run format/lint/targeted/full regression and shell guardrail evidence capture.

## Affected Modules
- `crates/kamn-core/src/data_layer_prd_critical_scenario_conformance.rs` (new)
- `crates/kamn-core/src/lib.rs` (module + exports)
- `crates/kamn-core/tests/data_layer_prd_critical_scenario_conformance.rs` (new)
- `specs/5028/spec.md`
- `specs/5028/plan.md`
- `specs/5028/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigations:
  - Hard-code required scenario catalog and keep output ordering deterministic.
  - Fail-closed on invalid IDs/duplicate mutation attempts.
  - Keep implementation Rust-only to avoid shell-surface growth.

## Interface Contract
- Additive public API under `kamn_core::data_layer_prd_critical_scenario_conformance::*`.
- No new dependencies.
- No protocol/wire-format changes.

## ADR
- Not required for this scoped additive implementation.

## Verification Summary
- RED: `cargo test -p kamn-core --test data_layer_prd_critical_scenario_conformance` (failed before implementation with unresolved `DataLayerPrdCriticalScenario*` symbols).
- GREEN: `cargo test -p kamn-core --test data_layer_prd_critical_scenario_conformance` (6 passed, 0 failed).
- Regression: `cargo test -p kamn-core` (pass), `cargo clippy -p kamn-core -- -D warnings` (pass), `cargo fmt --check` (pass).
