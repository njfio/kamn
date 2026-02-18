# Issue #4954 Tasks

- Issue: #4954
- Status: Implemented

## Ordered Tasks
- [x] T1: complete deletion-wave story (`#4955`) and child tasks/subtasks.
- [x] T2: complete archive-lifecycle story (`#4956`) and child tasks/subtasks.
- [x] T3: complete hard-ceiling/ratio-ratchet story (`#4957`) and child tasks/subtasks.
- [x] T4: synchronize epic lifecycle docs and closure evidence.

## Completion Evidence
- Stories delivered: `#4955` (closed), `#4956` (closed), `#4957` (closed)
- Representative governance suites:
  - `bash scripts/ci/test_check_superseded_script_deletion_manifest.sh`
  - `bash scripts/ci/test_check_stale_script_references.sh`
  - `bash scripts/ci/test_check_spec_archive_policy.sh`
  - `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh`
  - `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`
  - `bash scripts/ci/test_check_shell_surface_threshold_ratchet.sh`
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
