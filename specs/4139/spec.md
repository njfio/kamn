# Issue #4139 Spec

- Title: Subtask: add red fuzz-corpus drift tests and parser failure-taxonomy assertions
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
Deterministic fuzz harnesses exist, but corpus metadata and parser failure-taxonomy markers can still drift without explicit fail-closed regression contracts.

## Acceptance Criteria
- AC-1: Add red/green regression tests that fail closed on cargo-fuzz seed corpus metadata drift.
- AC-2: Add parser failure-taxonomy assertions for envelope and DID fuzz target metadata.
- AC-3: Add docs marker contract assertions for parser failure-taxonomy governance.
- AC-4: Targeted conformance/regression commands remain green.

## Scope
In scope:
- `crates/kamn-core/tests/cargo_fuzz_target_contract.rs`
- `crates/kamn-core/tests/invariant_and_fuzz_strategy_docs.rs`
- `docs/testing/invariant-and-fuzz-strategy.md`
- `specs/4139/{spec.md,plan.md,tasks.md}`

Out of scope:
- New fuzz targets or new shell runner scripts
- CI workflow expansion
- Production runtime behavior changes

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | cargo-fuzz replay metadata assertions | Fails closed when required seed files/target metadata drift |
| C-02 | AC-2 | Functional | parser failure-taxonomy marker assertions | Fails closed when expected taxonomy markers drift |
| C-03 | AC-3 | Conformance | docs marker assertions | Fails closed on missing/changed parser taxonomy contract markers |
| C-04 | AC-4 | Regression | targeted contract/doc tests | All targeted commands pass |

## Test Mapping
- `cargo test -p kamn-core --test cargo_fuzz_target_contract`
- `cargo test -p kamn-core --test invariant_and_fuzz_strategy_docs`
- `cargo fmt --check`

## Success Metrics
- Seed corpus metadata drift is surfaced by deterministic tests.
- Parser failure-taxonomy markers are explicitly contract-checked.
- Shell LOC growth remains zero for this subtask.
