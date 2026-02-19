# Issue #3936 Spec

- Title: Task: replace production expect/unreachable call paths with typed errors
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
Panic primitives in production startup/runtime paths (`expect(` and `unreachable!(`) violate fail-closed reliability expectations.

## Acceptance Criteria
- AC-1: Target production panic call sites are removed from scoped startup/runtime paths.
- AC-2: Typed errors remain observable through existing runtime/configuration channels.
- AC-3: Regression checks fail closed if panic primitives reappear in scoped production paths.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- Child subtask outputs:
  - `#3940` (`PR #5154`)
  - `#3941` (`PR #5153`)
- Runtime/docs guard artifacts:
  - `crates/kamn-node/src/cli_tests.rs`
  - `crates/kamn-node/src/signer.rs`
  - `docs/foundation/runtime-watchdog-attestation.md`

Out of scope:
- Broader panic-policy CI lane implementation (`#3937`)

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | startup/API/observability production-source panic scan | no production `expect(`/`unreachable!(`/`panic!(` in scoped files |
| C-02 | AC-2 | Unit | signer decode failure path | deterministic typed `ConfigError::RuntimeKolmeLive` handling |
| C-03 | AC-3 | Regression | signer-source and startup panic regressions | checks fail closed on reintroduction |
| C-04 | AC-4 | Integration | runtime watchdog docs contract + scoped lint/format | all mapped checks pass |

## Test Mapping
- `cargo test -p kamn-node regression_3940_production_source_extractor_retains_non_test_items`
- `cargo test -p kamn-node regression_3598_startup_paths_have_no_panic_control_flow`
- `cargo test -p kamn-node regression_signer_module_source_contains_no_unreachable_macro`
- `cargo test -p kamn-node regression_signer_private_key_decode_failure_redacts_sensitive_input`
- `cargo test -p kamn-core --test runtime_watchdog_attestation_docs`
- `cargo fmt --check`
- `cargo clippy -p kamn-node -- -D warnings`

## Success Metrics
- Panic primitives are retired from scoped production paths.
- Regression coverage now includes robust cfg(test)-aware extraction and signer-source macro guarding.
- Runtime watchdog docs capture #3940 and #3941 retirement mapping.
