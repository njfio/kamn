# Issue #4974 Spec

- Title: Subtask: implement specs archive tool and active-tree placement contract tests
- Status: Implemented
- Type: subtask
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
Issue #4974 requires a deterministic archive migration tool (not only passive checks) so implemented issue specs can be moved into `specs/archive/` with active-tree pointers and index parity enforced by contract tests.

## Acceptance Criteria
- AC-1: A deterministic archive tool exists to archive implemented issue specs and write active-tree pointers.
- AC-2: Archive tool emits deterministic reason-taxonomy markers and fails closed on invalid inputs/states.
- AC-3: Active-tree placement contract tests validate tool output against archive policy checker.
- AC-4: Issue/process/spec lifecycle artifacts are synchronized to implemented state.

## Scope
In scope:
- `scripts/ci/archive_completed_specs.py` tool implementation.
- Contract test expansion in `scripts/ci/test_check_spec_archive_policy.sh`.
- Archive policy checker parity validation against tool-generated output.

Out of scope:
- Bulk archival of new issue ranges beyond current wave.
- Non-spec archival policy dimensions.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run archive tool in dry-run and apply modes on synthetic implemented issue | Dry-run makes no moves; apply moves spec/plan/tasks and writes pointer/index |
| C-02 | AC-2 | Regression | Run archive tool with invalid/non-implemented inputs | Fail closed with deterministic reason codes |
| C-03 | AC-3 | Integration/Regression | Run archive policy checker against tool-generated fixture root | Checker reports `status=ok` with matching archive/pointer/index counts |
| C-04 | AC-4 | Functional/Regression | Verify issue + spec lifecycle markers | Issue body/process comments and `specs/4974/*` reflect implemented completion |

## Test Mapping
- AC-1:
  - `bash scripts/ci/test_check_spec_archive_policy.sh` (tool dry-run/apply coverage)
- AC-2:
  - `bash scripts/ci/test_check_spec_archive_policy.sh` (tool invalid-state fail assertions)
- AC-3:
  - `bash scripts/ci/test_check_spec_archive_policy.sh`
  - `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json /tmp/spec-archive-policy-report.json`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- AC-4:
  - `specs/4974/spec.md`
  - `specs/4974/plan.md`
  - `specs/4974/tasks.md`

## Success Metrics
- Archive migration tool exists and is executable with deterministic marker output.
- Active-tree placement contract tests pass against tool-generated fixtures.
- All ACs map to passing conformance/test evidence.
