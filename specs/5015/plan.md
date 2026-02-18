# Issue #5015 Plan

- Issue: #5015
- Status: Implemented

## Approach
1. Deliver story contracts through child task `#5028`:
   - PRD critical scenario catalog/evaluator for scenarios `62..71`,
   - shell-neutral orchestration policy enforcement,
   - fail-closed invalid ID and mutation guards.
2. Preserve additive exports for downstream M11 readiness/closure reporting.
3. Validate with scoped suite `data_layer_prd_critical_scenario_conformance` and crate-level regression.
4. Keep delivery Rust-only for this story to preserve shell budget neutrality.

## Affected Modules
- `crates/kamn-core/src/data_layer_prd_critical_scenario_conformance.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_prd_critical_scenario_conformance.rs`
- `specs/5015/spec.md`
- `specs/5015/plan.md`
- `specs/5015/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigations:
  - Keep required scenario ordering deterministic with stable reason markers.
  - Fail closed on invalid scenario IDs and mutating record attempts.
  - Preserve Rust-only delivery to avoid shell/workflow/python/template growth.

## Interface Contract
- Additive API exports in `kamn-core`.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this bounded additive story closure.

## Verification Summary
- RED: `cargo test -p kamn-core --test data_layer_prd_critical_scenario_conformance` (failed pre-implementation due to unresolved `DataLayerPrdCriticalScenario*` symbols in `#5028`).
- GREEN: `cargo test -p kamn-core --test data_layer_prd_critical_scenario_conformance` (6 passed, 0 failed).
- Regression: `cargo test -p kamn-core` (pass), `cargo clippy -p kamn-core -- -D warnings` (pass), `cargo fmt --check` (pass).
