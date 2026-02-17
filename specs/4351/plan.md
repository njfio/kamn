# Plan — #4351

Status: Reviewed

## Approach

- Add ratio-governance pass/fail assertions to:
  - `scripts/ci/test_check_kamn_core_rustdoc_artifact_policy.sh`
  - `scripts/ci/test_run_kamn_core_rustdoc_artifact_contract_lane.sh` (report marker presence)
- Run test first to capture RED before checker implementation.
