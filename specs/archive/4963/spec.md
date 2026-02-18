# Issue #4963 Spec

- Title: Task: execute initial completed-spec archival wave and regression validation
- Status: Implemented
- Type: task
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
After policy/tooling landed, the first migration wave needed execution evidence and parity checks to prevent stale/unsynchronized archive state.

## Acceptance Criteria
- AC-1: Initial archive migration wave produces traceable archive index/report artifacts.
- AC-2: Archive policy checker validates index/report parity deterministically.
- AC-3: Regression checks fail closed when archive index/report parity drifts.
- AC-4: Scoped archive-wave validation suites pass.

## Scope
In scope:
- Initial archive migration/report publication.
- Archive index/report parity enforcement in checker/tests.

Out of scope:
- Additional archival waves beyond first wave.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `specs/archive/index.md` | first-wave mapping/index present |
| C-02 | AC-2 | Regression | `check_spec_archive_policy.sh` index/report checks | deterministic GO when in parity |
| C-03 | AC-3 | Regression | parity-mutation fixtures | deterministic NO-GO reasons |
| C-04 | AC-4 | Integration | `bash scripts/ci/test_check_spec_archive_policy.sh` | wave+parity suite passes |

## Test Mapping
- AC-1/AC-2: `bash scripts/ci/test_check_spec_archive_policy.sh`
- AC-3: `bash scripts/ci/test_check_spec_archive_policy.sh` parity-failure cases
- AC-4: `bash scripts/ci/test_check_spec_archive_policy.sh`

## Success Metrics
- Archive-wave index/report published and policy-enforced.
- Parity regressions fail closed with deterministic reason taxonomy.
