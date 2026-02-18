# Issue #5027 Plan

- Issue: #5027
- Status: Implemented

## Approach
1. Add RED conformance tests (`spec_c01`..`spec_c06`) for:
   - deterministic hardening scenario registration and ordering,
   - readiness `Go` evaluation when all required scenarios pass,
   - readiness `NoGo` for critical failures and missing required outcomes,
   - duplicate scenario fail-closed guards and stable reason markers.
2. Implement `data_layer_m11_hardening_readiness` module with:
   - scenario domain/severity/status models,
   - deterministic registry and outcome recorder,
   - operator readiness evaluator and blocking reason projection.
3. Re-export M11 APIs in `crates/kamn-core/src/lib.rs`.
4. Run format/lint/targeted/full regression and shell guardrail evidence capture.

## Affected Modules
- `crates/kamn-core/src/data_layer_m11_hardening_readiness.rs` (new)
- `crates/kamn-core/src/lib.rs` (module + exports)
- `crates/kamn-core/tests/data_layer_m11_hardening_readiness.rs` (new)
- `specs/5027/spec.md`
- `specs/5027/plan.md`
- `specs/5027/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigations:
  - Enforce required-scenario and severity contracts with typed fail-closed errors.
  - Keep outputs deterministic (stable ordering + reason-code markers).
  - Keep implementation Rust-only to avoid shell-surface growth.

## Interface Contract
- Additive public API under `kamn_core::data_layer_m11_hardening_readiness::*`.
- No new dependencies.
- No protocol/wire-format changes.

## ADR
- Not required for this scoped additive implementation.

## Verification Summary
- RED: `cargo test -p kamn-core --test data_layer_m11_hardening_readiness` (failed before implementation with unresolved `DataLayerM11*` symbols).
- GREEN: `cargo test -p kamn-core --test data_layer_m11_hardening_readiness` (6 passed, 0 failed).
- Regression: `cargo test -p kamn-core` (pass), `cargo clippy -p kamn-core -- -D warnings` (pass), `cargo fmt --check` (pass).
