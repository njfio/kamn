# Issue #4975 Tasks

- Issue: #4975
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): capture failing behavior when archive index report is absent.
  Evidence:
  - `bash scripts/ci/check_spec_archive_policy.sh --repo-root <tmp-no-index> --output-json <tmp>` -> exit `1`, `reason_codes=spec_archive_index_missing`.
- [x] T2 (Green): publish first-wave archive mapping report and satisfy checker.
  Evidence:
  - Added `specs/archive/index.md` with deterministic wave/count markers and issue mappings.
  - `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json /tmp/spec-archive-policy-report.json` -> `status=ok`, `index_entry_count=9`.
- [x] T3 (Refactor): enforce index parity inside existing checker contract surface.
  Evidence:
  - `scripts/ci/check_spec_archive_policy.sh` now validates index presence, entry parity, and count parity.
- [x] T4 (Regression): extend and run checker contract tests.
  Evidence:
  - `bash scripts/ci/test_check_spec_archive_policy.sh` (includes missing-index fail case).
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`.
- [x] T5 (Docs): synchronize issue/process markers for shell-surface DoR and lifecycle logs.
  Evidence:
  - Issue `#4975` body updated with shell-surface estimates.
  - InProgress process comment posted on issue `#4975`.
- [x] T6 (Verify): finalize AC mapping and implemented lifecycle state.
  Evidence:
  - `specs/4975/spec.md`, `specs/4975/plan.md`, `specs/4975/tasks.md` set to Implemented.

## Completion Evidence
- Archive wave-1 mapping report is published and enforced by deterministic fail-closed policy checks.
