# Issue #3934 Spec

- Title: Story: eliminate production panic paths with typed fail-closed error handling
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
Production runtime paths must avoid panic primitives and remain fail-closed through typed error handling, checker enforcement, and docs-contract parity.

## Acceptance Criteria
- AC-1: Production `expect()`/`unreachable!()` panic paths in scoped runtime/entrypoint modules are removed or guarded by typed-error flows.
- AC-2: Error-path behavior and no-panic guarantees are covered by deterministic regression tests.
- AC-3: Panic-policy checker and docs-contract parity checks fail closed on drift.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- Child task outputs:
  - `#3936` (`PR #5155` closeout; implementation in `#5153` + `#5154`)
  - `#3937` (`PR #5158` closeout; implementation in `#5156` + `#5157`)
- Key runtime/docs artifacts:
  - `crates/kamn-node/src/signer.rs`
  - `crates/kamn-node/src/cli_tests.rs`
  - `scripts/ci/check_no_production_expect.py`
  - `scripts/ci/test_check_no_production_expect.sh`
  - `docs/foundation/runtime-watchdog-attestation.md`
  - `docs/ci/strategy.md`
  - `crates/kamn-core/tests/ci_strategy_docs.rs`

Out of scope:
- Non-runtime panic-policy expansion outside this milestone scope

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | startup/API/observability production-source panic scan + signer decode path | no scoped production panic primitives, typed signer error handling retained |
| C-02 | AC-2 | Regression | signer/startup regressions and cfg(test)-aware extraction checks | fail closed on panic primitive reintroduction |
| C-03 | AC-3 | Integration | panic checker harness + docs-contract parity tests | checker and docs drift guards fail closed and pass when aligned |
| C-04 | AC-4 | Regression | mapped test suites + guardrails from child tasks | all evidence remains green |

## Test Mapping
- `cargo test -p kamn-node regression_signer_module_source_contains_no_unreachable_macro`
- `cargo test -p kamn-node regression_signer_private_key_decode_failure_redacts_sensitive_input`
- `cargo test -p kamn-node regression_3598_startup_paths_have_no_panic_control_flow`
- `cargo test -p kamn-node regression_3940_production_source_extractor_retains_non_test_items`
- `bash scripts/ci/test_check_no_production_expect.sh`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_panic_path_policy_checker_markers_and_remediation_parity -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs`
- shell guardrail checks run in `#3942` (`hard ceiling`, `ratio`, `threshold ratchet`)

## Success Metrics
- Panic-path retirement and checker/docs governance are closed across both child tasks.
- Story-level AC/conformance mapping is fully traceable and implemented.
