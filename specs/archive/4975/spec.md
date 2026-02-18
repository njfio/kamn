# Issue #4975 Spec

- Title: Subtask: run first archive migration wave and publish archived-spec index report
- Status: Implemented
- Type: subtask
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
Issue #4975 requires publishing a deterministic index report for the first archive migration wave and enforcing that index stays synchronized with archived spec entries and active-tree pointers.

## Acceptance Criteria
- AC-1: First-wave archived spec index mapping report is published at `specs/archive/index.md` with deterministic markers and issue-to-path mappings.
- AC-2: Archive policy checker fails closed when the index report is missing, out of sync, or count-mismatched.
- AC-3: Archive policy checker and contract tests pass for the repository state after publishing the index.
- AC-4: Issue/process/spec lifecycle artifacts are synchronized to implemented state.

## Scope
In scope:
- `specs/archive/index.md` wave-1 mapping report publication.
- `scripts/ci/check_spec_archive_policy.sh` index parity enforcement.
- `scripts/ci/test_check_spec_archive_policy.sh` regression coverage for missing index report.

Out of scope:
- Archiving additional issue ranges beyond the first migration wave already moved into `specs/archive/`.
- Broader archive policy redesign.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Inspect `specs/archive/index.md` | Contains wave markers, count marker, and entries for archived issues 4859/4860/4861/4862/4863/4866/4867/4868/4869 |
| C-02 | AC-2 | Regression | Run checker on fixture root with archived entries but no `specs/archive/index.md` | Fails with `spec_archive_index_missing` |
| C-03 | AC-3 | Unit/Integration | Run checker + policy contract tests on repository | `status=ok`, count/pointer/index parity markers reported |
| C-04 | AC-4 | Functional/Regression | Verify issue/spec lifecycle markers | Issue body/process log + `specs/4975/*` reflect implemented completion |

## Test Mapping
- AC-1:
  - `specs/archive/index.md`
- AC-2:
  - `bash scripts/ci/check_spec_archive_policy.sh --repo-root <tmp-no-index> --output-json <tmp>`
- AC-3:
  - `bash scripts/ci/test_check_spec_archive_policy.sh`
  - `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json /tmp/spec-archive-policy-report.json`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- AC-4:
  - `specs/4975/spec.md`
  - `specs/4975/plan.md`
  - `specs/4975/tasks.md`

## Success Metrics
- Archived issue index report for wave-1 is published and deterministic.
- Archive checker surfaces `index_entry_count` and enforces report presence/parity.
- All ACs map to passing conformance/test evidence.
