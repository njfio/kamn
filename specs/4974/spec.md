# Issue #4974 Spec

- Title: Subtask: implement specs archive tool and active-tree placement contract tests
- Status: Reviewed
- Type: subtask
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
Issue #4974 is part of the R27.44 shell maintainability tranche and closes one portion of the deletion-wave, spec-archival, and hard-ceiling governance gap.

## Acceptance Criteria
- AC-1: Scope defined in GitHub issue #4974 is implemented and verified.
- AC-2: Deterministic fail-closed behavior is preserved for drift/regression scenarios.
- AC-3: Required Unit/Functional/Integration/Regression tests are present and passing.
- AC-4: Documentation/process markers remain synchronized where issue scope requires docs updates.

## Scope
In scope:
- Work explicitly described in issue #4974.

Out of scope:
- Unrelated feature expansion outside the issue boundary.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Execute scoped workflow for #4974 | Behavior matches issue acceptance criteria |
| C-02 | AC-2 | Regression | Inject marker/schema/taxonomy drift scenario | Policy/output fails closed with deterministic reasons |
| C-03 | AC-3 | Unit/Integration | Run scoped tests and lane checks | Required suites pass |
| C-04 | AC-4 | Functional/Regression | Validate docs/process marker contract checks | Marker parity remains verified |

## Test Mapping
- To be completed during implementation for issue #4974.

## Success Metrics
- All ACs for #4974 are mapped to conformance cases and passing tests.
- No shell-surface governance regressions introduced by #4974.
