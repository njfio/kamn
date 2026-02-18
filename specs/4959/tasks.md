# Issue #4959 Tasks

- Issue: #4959
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): prove deletion-manifest drift fails deterministically.
  - Evidence: legacy inventory vs expanded manifest -> `superseded_deletion_manifest_references_unknown_script`.
- [x] T2 (Green): execute runtime/kolme wave activation and wrapper compaction (`#4970` / PR `#4985`).
- [x] T3 (Green): execute canary/ci/deploy/governance wave activation (`#4969` / PR `#4992`).
- [x] T4 (Regression): keep stale-reference and command-surface checks green after wave expansion.
- [x] T5 (Docs/Process): synchronize parent-task lifecycle docs + closure markers.
- [x] T6 (Verify): run scoped suites and keep CI fast gate green before merge.

## Completion Evidence
- `bash scripts/ci/test_check_superseded_script_deletion_manifest.sh`
- `bash scripts/ci/test_check_stale_script_references.sh`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
- Subtask merges: PR `#4985` and PR `#4992`
