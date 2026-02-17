# Tasks — Issue #4821

- [x] T1 (Red): update manifest-backed dispatcher matrix test to require manifest `wrapper_name` and `phase`; capture failing output prior to migration.
- [x] T2 (Green): add metadata-driven resolver and migrate non-Kolme manifests with `wrapper_name`/`phase`.
- [x] T3 (Refactor): remove hardcoded dispatcher wrapper/phase case statements.
- [x] T4 (Verify): run matrix/deep dispatch suites and record deterministic evidence.

## Verification Evidence

- RED:
  - `bash scripts/framework/test_non_kolme_manifest_backed_contract_lane_dispatch_wrapper_matrix.sh`
  - failure excerpt: `expected wrapper_name='run_launch_canary_contract_lane.sh' in canary_launch_canary_contract_lane.json, got None`
- GREEN:
  - `bash scripts/framework/test_non_kolme_manifest_backed_contract_lane_dispatch_wrapper_matrix.sh`
  - `for t in scripts/framework/test_non_kolme*contract_lane_dispatch_wrapper_matrix.sh; do bash \"$t\"; done`
  - `bash scripts/bridge/test_bridge_deep_lane_dispatch_wrapper_matrix.sh`
