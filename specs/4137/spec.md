# Issue #4137 Spec

- Title: Subtask: split monolithic test suites into domain modules with parity-preserving red tests
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
`task_escrow_proptest_invariants.rs` combines task and escrow property domains in one dense integration-test file, making maintenance and targeted evolution harder.

## Acceptance Criteria
- AC-1: Split `task_escrow_proptest_invariants` into domain modules (task + escrow) with shared helpers.
- AC-2: Add red-first regression/contract checks that fail if module split/parity wiring drifts.
- AC-3: Preserve existing test names/behavioral coverage parity after split.
- AC-4: Document suite modularization conventions in testing docs.
- AC-5: Targeted suites remain green and deterministic.

## Scope
In scope:
- `crates/kamn-core/tests/task_escrow_proptest_invariants.rs`
- `crates/kamn-core/tests/task_escrow_proptest_invariants/{shared.rs,task_domain.rs,escrow_domain.rs}` (new)
- `crates/kamn-core/tests/task_escrow_suite_modularization_contract.rs` (new)
- `docs/testing/strategy.md` (new)
- `specs/4137/{spec.md,plan.md,tasks.md}`

Out of scope:
- Production runtime behavior changes
- Fuzz/concurrency lane redesign
- New dependencies

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | Compile/load split task+escrow modules via root test harness | Domain modules load through root module declarations |
| C-02 | AC-2 | Regression | Module split contract test | Fails closed on missing declarations/files |
| C-03 | AC-3 | Regression | `task_escrow_proptest_invariants` full run | Existing test names and deterministic behavior remain |
| C-04 | AC-4 | Functional | Testing strategy docs check | Modularization conventions documented |
| C-05 | AC-5 | Regression | full targeted suite + fmt/clippy/guardrails | All pass |

## Test Mapping
- `cargo test -p kamn-core --test task_escrow_suite_modularization_contract`
- `cargo test -p kamn-core --test task_escrow_proptest_invariants`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`

## Success Metrics
- Monolithic property suite is decomposed by domain with shared helper layer.
- Split wiring is protected by deterministic contract checks.
- Deterministic property behavior remains stable.
