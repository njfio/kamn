# Issue #3938 Spec

- Title: Task: split node main test surfaces into focused modules with behavior parity
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
Node runtime test surfaces risk regressing into monoliths and command-surface drift without deterministic decomposition and parity contracts.

## Acceptance Criteria
- AC-1: Oversized runtime test surface is decomposed into focused module files with stable selector behavior.
- AC-2: Command-surface parity checks fail closed on selector/docs drift.
- AC-3: Ownership and structural-budget boundaries are contract-tested.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- `crates/kamn-node/src/main_tests/runtime_tests.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/*.rs`
- `crates/kamn-node/tests/main_module_extraction_contract.rs`
- `crates/kamn-node/tests/main_tests_command_surface_parity_contract.rs`
- `docs/ci/strategy.md`
- `docs/foundation/runtime-watchdog-attestation.md`
- `specs/3944/*`
- `specs/3945/*`

Out of scope:
- Runtime production behavior changes.
- CI workflow or shell lane expansion.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | runtime test module extraction (`#3944`) | `runtime_tests.rs` is bounded include-shell with focused modules |
| C-02 | AC-2 | Functional | command-surface parity contract (`#3945`) | selector symbol/docs drift fails closed with deterministic markers |
| C-03 | AC-3 | Regression | extraction and parity contract suites | ownership/budget and command-surface boundaries remain enforced |
| C-04 | AC-4 | Integration | combined `kamn-node`/docs contract runs | mapped suites pass |

## Test Mapping
- `cargo test -p kamn-node --test main_module_extraction_contract`
- `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`
- `cargo test -p kamn-core --test ci_strategy_docs`
- `cargo test -p kamn-core --test runtime_watchdog_attestation_docs`

## Success Metrics
- Runtime test-surface decomposition remains stable and maintainable.
- Command-surface parity drift is blocked before merge.
