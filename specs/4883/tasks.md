# Tasks — Issue #4883

- [x] T1 (Red): execute lane-registry drift contracts and capture failing stale-artifact signal.
- [x] T2 (Green): synchronize stale registry manifest payload metadata (`wrapper_name`, `phase`) for migrated Kolme lanes.
- [x] T3 (Refactor): preserve registry-driven generation as the single maintenance path and remove manual-field drift.
- [x] T4 (Verify): re-run framework generation/drift contracts and integration test harness coverage.

## Verification Evidence

- Red evidence (before fix): `python3 scripts/framework/generate_lane_artifacts.py --registry-file scripts/framework/lane_registry.json --repo-root . --mode check` -> `status=fail`, `error=manifest drift detected: 59 entries`.
- `bash scripts/framework/test_check_lane_registry_drift.sh` -> `lane registry drift checker tests passed.`
- `bash scripts/framework/test_lane_registry_generation.sh` -> `lane registry generation tests passed.`
- `bash scripts/framework/test_contract_framework.sh` -> `contract framework tests passed.`
