# Issue #4135 Spec

- Title: Subtask: add red proptest cases for transition legality and invariant preservation
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
Existing proptest lanes cover task/escrow and peer lifecycle transitions, but `#4135` remains open without explicit conformance coverage for transition-evidence legality invariants and without documentation markers for the invariant policy lane.

## Acceptance Criteria
- AC-1: Add deterministic proptest coverage for task transition evidence legality/invariant preservation.
- AC-2: Add deterministic proptest coverage for escrow transition evidence legality/invariant preservation.
- AC-3: Add deterministic proptest coverage for peer invalid-transition rejection invariants and reason-code stability.
- AC-4: Update invariant/fuzz testing documentation markers to reflect the transition-legality property lane.
- AC-5: All relevant proptest suites remain green and deterministic.

## Scope
In scope:
- `crates/kamn-core/tests/task_escrow_proptest_invariants.rs`
- `crates/kamn-core/tests/peer_lifecycle_proptest_invariants.rs`
- `docs/testing/invariant-and-fuzz-strategy.md`
- `specs/4135/{spec.md,plan.md,tasks.md}`

Out of scope:
- Runtime/escrow/task production behavior redesign
- New dependencies
- Shell/workflow/tooling expansion

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Randomized task transition sequences with evidence API | Successful evidence matches legal graph and state history invariants |
| C-02 | AC-2 | Functional | Randomized escrow transition actions via evidence API | Amount-conservation/status invariants hold and evidence reason code is stable |
| C-03 | AC-3 | Functional | Randomized peer event sequences | Illegal transitions are rejected with deterministic reason code and no state drift |
| C-04 | AC-4 | Regression | Testing strategy docs marker assertions | Required transition-legality property markers are present |
| C-05 | AC-5 | Regression | Full targeted property suites | All tests pass with deterministic configuration |

## Test Mapping
- `cargo test -p kamn-core --test task_escrow_proptest_invariants`
- `cargo test -p kamn-core --test peer_lifecycle_proptest_invariants`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`

## Success Metrics
- `#4135` closure no longer depends on manual interpretation of existing tests.
- Deterministic property contracts explicitly cover evidence-oriented legality invariants.
- Documentation markers for invariant property policy stay synchronized with tests.
