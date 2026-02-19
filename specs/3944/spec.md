# Issue #3944 Spec

- Title: Subtask: extract node test suites into focused module files with ownership boundaries
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
`crates/kamn-node/src/main_tests/runtime_tests.rs` has regrown into a large monolith, which increases maintenance cost and makes targeted ownership boundaries hard to enforce.

## Acceptance Criteria
- AC-1: `runtime_tests.rs` becomes a bounded shell that delegates test bodies to focused module files under `src/main_tests/runtime_tests/`.
- AC-2: Existing test selector paths remain stable (`main_tests::runtime_tests::<test_name>`) after extraction.
- AC-3: Extraction ownership and bounded-shell constraints fail closed via deterministic contract assertions.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- `crates/kamn-node/src/main_tests/runtime_tests.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/*.rs`
- `crates/kamn-node/tests/main_module_extraction_contract.rs`
- `docs/foundation/runtime-watchdog-attestation.md`
- `specs/3944/spec.md`
- `specs/3944/plan.md`
- `specs/3944/tasks.md`

Out of scope:
- Renaming existing runtime test function selectors.
- CI workflow wiring changes.
- Service API or signer test-surface decomposition outside runtime tests.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `src/main_tests/runtime_tests.rs` | file contains include-based shell wiring and no inline runtime test bodies |
| C-02 | AC-2 | Integration | targeted runtime test selectors from existing docs/scripts | selectors resolve and execute with unchanged names |
| C-03 | AC-3 | Regression | `main_module_extraction_contract` runtime shell checks | fails if `runtime_tests.rs` regresses into inline monolith |
| C-04 | AC-4 | Regression | targeted `kamn-node` + contract test runs | all mapped checks pass |

## Test Mapping
- `cargo test -p kamn-node --test main_module_extraction_contract main_module_extraction_contract_runtime_tests_decomposition_shell_markers_remain_stable -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::functional_runtime_kolme_live_retries_transient_submit_and_finality_unavailable_errors -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::integration_runtime_full_emits_ordered_bootstrap_readiness_markers -- --exact`
- `cargo test -p kamn-node --test main_module_extraction_contract`

## Success Metrics
- `runtime_tests.rs` shrinks to shell-only ownership wiring.
- Runtime test command-surface remains stable (no selector drift).
- Contract guard fails closed on shell/boundary regressions.
