# Tasks - Issue #4163

- [x] T1 (Red): add failing tests for rotation preflight quorum marker parity/tamper rejection (`#4169`).
- [x] T2 (Green): implement custody reason mapping and evidence marker checks (`#4170`).
- [x] T3 (Refactor/Docs): keep deployment/release docs contracts aligned with rotation marker governance.
- [x] T4 (Verify): re-run preflight policy/lane and docs-contract checks for parent closure.

## Planned Verification Commands

- `bash scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh`
- `bash scripts/kolme/test_run_local_kolme_live_deployment_preflight_lane.sh`
- `cargo test -p kamn-core --test kolme_devnet_ops_docs`
- `cargo test -p kamn-core --test release_gonogo_checklist_docs`
