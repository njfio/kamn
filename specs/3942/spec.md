# Issue #3942 Spec

- Title: Subtask: implement scoped panic-path policy checker for production files
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
The panic-path policy checker currently truncates scanning at the first `#[cfg(test)]` line, which can miss production violations in files that declare test-only imports near the top.

## Acceptance Criteria
- AC-1: Checker scans production code after top-level `#[cfg(test)]` attributes while still excluding test-only items.
- AC-2: Checker fail-closed reason taxonomy remains deterministic for panic primitives and unsafe fallback defaults.
- AC-3: Checker regression harness includes a fixture proving post-`#[cfg(test)]` production violations are detected.
- AC-4: Unit/Functional/Integration/Regression evidence is present and passing.

## Scope
In scope:
- `scripts/ci/check_no_production_expect.py`
- `scripts/ci/test_check_no_production_expect.sh`
- `specs/3942/spec.md`
- `specs/3942/plan.md`
- `specs/3942/tasks.md`

Out of scope:
- Workflow wiring/docs-contract parity (`#3943`)
- Broader static-analysis replacement beyond this checker

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | source fixture with top-level `#[cfg(test)] use` then production `expect(` | checker reports `production_expect_reachable` |
| C-02 | AC-1 | Regression | source fixture with test-only module using `expect(` | checker remains pass for test-only panic usage |
| C-03 | AC-2 | Functional | panic/unsafe fallback fixtures | deterministic reason codes and reason class values |
| C-04 | AC-3 | Integration | `scripts/ci/test_check_no_production_expect.sh` | full checker test harness passes including new fixture |

## Test Mapping
- `bash scripts/ci/test_check_no_production_expect.sh`
- `python3 scripts/ci/check_no_production_expect.py --root <tmp-fixture-root>`

## Success Metrics
- No false negatives for production violations after top-level test cfg attributes.
- Existing deterministic reason taxonomy outputs remain unchanged.
