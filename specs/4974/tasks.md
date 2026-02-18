# Issue #4974 Tasks

- Issue: #4974
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): add failing/negative archive-tool contract assertions.
  Evidence:
  - `bash scripts/ci/test_check_spec_archive_policy.sh` exercises missing-index and missing-pointer fail-closed paths and validates deterministic reason markers.
- [x] T2 (Green): implement archive migration tool with deterministic outputs.
  Evidence:
  - Added executable `scripts/ci/archive_completed_specs.py`.
  - Tool supports dry-run/apply and emits stable marker/report fields.
- [x] T3 (Refactor): keep archive index and pointer writing deterministic.
  Evidence:
  - Tool writes canonical `ARCHIVED.md` pointer content and deterministic archive index row format.
- [x] T4 (Regression): expand active-tree placement contract coverage.
  Evidence:
  - `scripts/ci/test_check_spec_archive_policy.sh` now validates tool dry-run/apply behavior and checker acceptance of tool-generated fixture output.
- [x] T5 (Docs): synchronize issue/process lifecycle markers.
  Evidence:
  - Updated issue body with shell-surface estimate markers.
  - Posted InProgress process log comment on issue `#4974`.
- [x] T6 (Verify): run scoped checks and finalize lifecycle artifacts.
  Evidence:
  - `python3 -m py_compile scripts/ci/archive_completed_specs.py`
  - `bash scripts/ci/test_check_spec_archive_policy.sh`
  - `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json /tmp/spec-archive-policy-report.json`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
  - `specs/4974/spec.md`, `specs/4974/plan.md`, `specs/4974/tasks.md` set to Implemented.

## Completion Evidence
- Archive migration tool and active-tree placement contract tests are implemented and passing with deterministic fail-closed behavior.
