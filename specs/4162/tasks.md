# Tasks - Issue #4162

- [x] T1 (Red): add failing tests for fallback signer-key prohibition and explicit signer-material requirements (`#4167`).
- [x] T2 (Green): implement deterministic missing/invalid signer-material error mapping and fallback removal (`#4168`).
- [x] T3 (Refactor/Docs): keep ops docs and docs-contract markers aligned with explicit signer-material requirements.
- [x] T4 (Verify): re-run preflight policy, preflight lane, and docs-contract tests for parent closure.

## Planned Verification Commands

- `bash scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh`
- `bash scripts/kolme/test_run_local_kolme_live_deployment_preflight_lane.sh`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs`
