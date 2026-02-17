# Spec — Issue #4826

- Title: Subtask: add JSON emit/write helpers and migrate top manual JSON construction scripts
- Parent: #4814
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Introduce shared JSON write helpers and migrate the high-volume ROOT_DIR-based shell JSON heredoc writers to helper-based emission so JSON output handling is centralized and deterministic.

## Problem Statement

Manual `cat <<...` JSON block writes are duplicated across many shell scripts, inflating maintenance cost and creating repeated JSON emission logic.

## Scope

In scope:
- Add shared JSON helper primitives in `scripts/lib/common.sh`.
- Add executable helper command `scripts/lib/write_json_file.sh`.
- Migrate detected ROOT_DIR-based manual JSON heredoc writers to helper command invocation.
- Add migration contract test to enforce helper presence and migrated-surface expectations.

Out of scope:
- Non-shell JSON emitters.
- Full declarative policy-checker migration (covered by later milestones/subtasks).

## Acceptance Criteria

- AC-1: Shared JSON helper primitives exist in `scripts/lib/common.sh` and executable helper command exists in `scripts/lib/write_json_file.sh`.
- AC-2: Manual ROOT_DIR-based JSON heredoc writers are migrated to helper invocations across the targeted high-duplication cohort.
- AC-3: Contract and regression suites pass with no behavior regressions in CI tools.

## Conformance Cases

- C-01 (AC-1): `bash scripts/lib/test_json_write_helper_migration_contract.sh` passes helper existence checks.
- C-02 (AC-2): migration contract confirms at least 80 helper-adopted scripts and zero remaining ROOT_DIR-based `cat` JSON heredoc writers.
- C-03 (AC-3): `bash scripts/ci/test_ci_tools.sh` passes after migration.

## Success Metrics / Signals

- 90 shell scripts reference `scripts/lib/write_json_file.sh` after migration.
- 89 scripts (168 JSON heredoc write sites) migrated in this tranche.
- ROOT_DIR-based manual `cat` JSON heredoc count reduced to 0.
