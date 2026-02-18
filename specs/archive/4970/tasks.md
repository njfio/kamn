# Issue #4970 Tasks

- Issue: #4970
- Status: Completed

## Ordered Tasks
- [x] T1 (Red): add failing tests derived from issue ACs and conformance cases.
- [x] T2 (Green): implement minimum change to satisfy tests deterministically.
- [x] T3 (Refactor): simplify and harden without changing behavior.
- [x] T4 (Regression): add drift/tamper/marker parity regression checks.
- [x] T5 (Docs): update required docs/process markers for issue #4970.
- [x] T6 (Verify): run scoped unit/functional/integration/regression checks and record evidence.

## Completion Evidence
- `bash scripts/ci/test_check_stale_script_references.sh`
- `bash scripts/ci/test_check_superseded_script_deletion_manifest.sh`
- `bash scripts/kolme/test_contract_lane_dispatch_wrapper_compaction.sh`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- `bash scripts/ci/test_kolme_command_surface_asymmetry_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
