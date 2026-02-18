# Tasks — Issue #4860

- [x] T1 (Red): phase-0 and phase-1/2 contract tests added to fail on helper boilerplate drift and dispatcher metadata-resolution regressions.
- [x] T2 (Green): migrated helper boilerplate to `common.sh` and replaced hardcoded dispatcher maps with manifest-driven metadata resolution.
- [x] T3 (Refactor): reduced duplicated shell helper/dispatch logic while preserving deterministic fallback reason taxonomy markers.
- [x] T4 (Verify): child task deliveries merged and story-level evidence consolidated.

## Verification Evidence

- Child task PRs: `#4889` (task #4864), `#4890` (task #4865).
- `bash scripts/kolme/test_common_sh_helper_migration_contract.sh` → migration contract passes with `status=ok`.
- `bash scripts/kolme/test_dispatcher_manifest_metadata_contract.sh` → dispatcher metadata contract passes.
- `bash scripts/kolme/test_contract_lane_dispatch_wrapper_matrix.sh` → wrapper matrix regression passes.
- `bash scripts/kolme/test_run_local_kolme_api_smoke_lane.sh` → integration lane regression passes.
- `bash scripts/ci/test_kolme_command_surface_asymmetry_contract.sh` and `bash scripts/ci/test_kolme_command_surface_coverage_contract.sh` → command-surface governance regressions pass.
