# Issue #4973 Spec

- Title: Subtask: define archive layout and completed-spec retention policy markers
- Status: Implemented
- Type: subtask
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
Issue #4973 requires explicit, stable governance markers for archive layout and retention so archival behavior is policy-driven and contract-testable instead of implied by scattered implementation details.

## Acceptance Criteria
- AC-1: Archive layout and retention policy is documented with deterministic marker lines.
- AC-2: Marker presence is enforced by contract tests (fail when required markers are removed).
- AC-3: Existing archive policy checker/test suite remains green with the new policy contract.
- AC-4: Issue/process/spec lifecycle artifacts are synchronized to implemented state.

## Scope
In scope:
- `docs/planning/spec-archive-policy.md` policy marker documentation.
- Marker contract assertions in `scripts/ci/test_check_spec_archive_policy.sh`.
- Milestone marker linkage update for archive policy doc.

Out of scope:
- Changing archive retention decisions for already-archived issue sets.
- Non-spec retention domains.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Inspect policy doc | Required archive layout/retention markers are present |
| C-02 | AC-2 | Regression | Remove/alter required marker in policy doc fixture | `test_check_spec_archive_policy.sh` fails with marker-missing assertion |
| C-03 | AC-3 | Unit/Integration | Run archive policy checker/contract tests | Archive policy suite remains green |
| C-04 | AC-4 | Functional/Regression | Verify issue/spec/process markers | Issue body/process comments and `specs/4973/*` are synchronized |

## Test Mapping
- AC-1:
  - `docs/planning/spec-archive-policy.md`
- AC-2:
  - `bash scripts/ci/test_check_spec_archive_policy.sh` (required marker assertions)
- AC-3:
  - `bash scripts/ci/test_check_spec_archive_policy.sh`
  - `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json /tmp/spec-archive-policy-report.json`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- AC-4:
  - `specs/4973/spec.md`
  - `specs/4973/plan.md`
  - `specs/4973/tasks.md`

## Success Metrics
- Archive policy markers are centralized in one governance doc and enforced in CI contract tests.
- All ACs map to passing conformance/test evidence.
