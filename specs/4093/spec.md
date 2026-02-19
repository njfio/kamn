# Issue #4093 Spec

- Title: Subtask: add docs parity and remediation marker checks for fairness governance
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Problem Statement
Fairness governance contracts lose value when docs and remediation markers drift away from checker reason taxonomy. We need fail-closed parity checks that keep `docs/ci/strategy.md`, `docs/ops/configuration.md`, fixtures, and checker outputs synchronized.

## Acceptance Criteria
- AC-1: `docs/ci/strategy.md` defines deterministic fairness docs-parity markers (taxonomy version, reason-code CSV, referenced docs/fixture paths).
- AC-2: Remediation markers in `docs/ci/strategy.md` are present for every fairness reason code exposed by the checker.
- AC-3: Parity checks fail closed when docs markers drift from checker reason taxonomy or fixture metadata.
- AC-4: Unit, Functional, Integration, and Regression tests cover fairness docs parity/remediation contracts and pass.

## Scope
In scope:
- Add fairness docs-parity/remediation marker block to `docs/ci/strategy.md`.
- Add Rust tests that compare checker reason taxonomy with docs markers and fixture metadata.
- Add regression checks that require per-reason remediation markers.
- Create `specs/4093/{spec.md,plan.md,tasks.md}`.

Out of scope:
- New shell/python/workflow checkers.
- Scheduler/runtime fairness algorithm changes.
- CI lane topology changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `docs/ci/strategy.md` fairness docs-parity section | Required taxonomy/csv/path markers are present and deterministic |
| C-02 | AC-2 | Unit | Fairness checker reason-code CSV | One remediation marker exists per reason code |
| C-03 | AC-3 | Integration | Checker taxonomy + fixture metadata + docs markers | Reason taxonomy/csv remain synchronized; drift fails closed |
| C-04 | AC-4 | Regression | Marker-drift assertions | Missing/changed fairness marker contracts fail test deterministically |

## Test Mapping
- `cargo test -p kamn-core --test fairness_docs_parity_contract`
- `cargo test -p kamn-core --test ci_strategy_docs -- doc_contains_fairness_docs_parity_and_remediation_markers --exact`

## Success Metrics
- Fairness docs parity and remediation mapping are enforced by deterministic Rust tests.
- No shell LOC growth for this subtask (`shell_loc_delta_actual=0` target).
