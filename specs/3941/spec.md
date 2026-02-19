# Issue #3941 Spec

- Title: Subtask: replace production unreachable!() branches with explicit typed errors
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
`unreachable!()` in production-oriented modules creates panic-capable drift risk and conflicts with the fail-closed typed-error policy.

## Acceptance Criteria
- AC-1: Startup/runtime signer paths do not contain `unreachable!()` in production code regions.
- AC-2: Decode-failure handling remains typed (`ConfigError::RuntimeKolmeLive`) and deterministic.
- AC-3: Regression tests fail closed if `unreachable!()` is reintroduced in signer module source.
- AC-4: Unit/Functional/Integration/Regression evidence for this subtask is present and passing.

## Scope
In scope:
- `crates/kamn-node/src/signer.rs`
- `specs/3941/spec.md`
- `specs/3941/plan.md`
- `specs/3941/tasks.md`

Out of scope:
- Broader panic-path policy lane implementation (`#3937`, `#3942`, `#3943`)
- Node test-surface modularization (`#3938`, `#3939`)

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | inspect production segment of signer source | no `unreachable!()` in production region |
| C-02 | AC-2 | Unit | invalid signer private-key decode path | deterministic `ConfigError::RuntimeKolmeLive` without panic macro control-flow |
| C-03 | AC-3 | Regression | signer module source regression test | test fails if `unreachable!()` appears in signer source |
| C-04 | AC-4 | Integration | run scoped `kamn-node` test suite | signer + startup panic-path regression checks pass together |

## Test Mapping
- `cargo test -p kamn-node regression_signer_module_source_contains_no_unreachable_macro -- --exact`
- `cargo test -p kamn-node regression_signer_private_key_decode_failure_redacts_sensitive_input -- --exact`
- `cargo test -p kamn-node regression_3598_startup_paths_have_no_panic_control_flow -- --exact`

## Success Metrics
- `unreachable!()` is absent from `crates/kamn-node/src/signer.rs`.
- Decode failure path remains typed and deterministic.
- All mapped tests pass without introducing shell-surface growth.
