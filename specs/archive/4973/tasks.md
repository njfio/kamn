# Issue #4973 Tasks

- Issue: #4973
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): add marker-presence assertions for archive policy docs.
  Evidence:
  - `scripts/ci/test_check_spec_archive_policy.sh` now fails when required archive policy marker lines are missing.
- [x] T2 (Green): publish archive policy marker doc.
  Evidence:
  - Added `docs/planning/spec-archive-policy.md` with deterministic layout/retention markers.
- [x] T3 (Refactor): align milestone marker linkage with policy doc.
  Evidence:
  - Added `spec_archive_policy_doc=docs/planning/spec-archive-policy.md` to milestone governance marker list.
- [x] T4 (Regression): run archive policy checker/test contracts.
  Evidence:
  - `bash scripts/ci/test_check_spec_archive_policy.sh`
  - `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json /tmp/spec-archive-policy-report.json`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- [x] T5 (Docs): synchronize issue/process lifecycle markers.
  Evidence:
  - Issue body updated with shell-surface estimate markers.
  - InProgress process log comment posted on issue `#4973`.
- [x] T6 (Verify): finalize AC mapping and implemented lifecycle state.
  Evidence:
  - `specs/4973/spec.md`, `specs/4973/plan.md`, `specs/4973/tasks.md` set to Implemented.

## Completion Evidence
- Archive layout and retention markers are documented and enforced by contract tests.
