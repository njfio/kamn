# Issue #3940 Spec

- Title: Subtask: migrate production expect() callsites to typed error propagation
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
Production expect-path checks can be bypassed when source scanning truncates at the first `#[cfg(test)]` attribute, especially in files that declare test-only imports near the top.

## Acceptance Criteria
- AC-1: Production-source extraction used by panic-path regression checks correctly skips test-only items without truncating later production code.
- AC-2: Startup/API/observability runtime source checks fail closed on `expect(`, `unreachable!(`, and `panic!(` in production regions.
- AC-3: Panic-path retirement mapping documents the #3940 expect-callsite guard behavior.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- `crates/kamn-node/src/cli_tests.rs`
- `docs/foundation/runtime-watchdog-attestation.md`
- `specs/3940/spec.md`
- `specs/3940/plan.md`
- `specs/3940/tasks.md`

Out of scope:
- CI checker script redesign (`scripts/ci/check_no_production_expect.py`)
- Non-runtime crates beyond current node startup/API/observability scope

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | synthetic source with top-level `#[cfg(test)] use` followed by production function | extractor retains production function text after test-only import |
| C-02 | AC-2 | Functional | panic-path regression over startup/API/observability source list | no production `expect(` / `unreachable!(` / `panic!(` occurrences |
| C-03 | AC-3 | Regression | runtime watchdog attestation docs contract | panic-path retirement mapping includes #3940 guard statement |
| C-04 | AC-4 | Integration | mapped node/core tests + lint/format | all checks pass |

## Test Mapping
- `cargo test -p kamn-node regression_3940_production_source_extractor_retains_non_test_items -- --exact`
- `cargo test -p kamn-node regression_3598_startup_paths_have_no_panic_control_flow -- --exact`
- `cargo test -p kamn-core --test runtime_watchdog_attestation_docs`
- `cargo fmt --check`
- `cargo clippy -p kamn-node -- -D warnings`

## Success Metrics
- Production-source extractor no longer false-negatives files with top-level test cfg attributes.
- Regression coverage explicitly includes API/observability runtime entrypoint modules.
- Runtime watchdog docs include #3940 panic-path retirement mapping.
