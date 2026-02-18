# Tasks - Issue #4159

- [x] T1 (Red): add failing tests for signer zeroization and fallback-prohibition contracts (`#4161`, `#4162`).
- [x] T2 (Green): implement signer zeroization controls and deterministic explicit signer-material mapping.
- [x] T3 (Refactor/Docs): align ops/docs contracts with zeroization and explicit-material policy markers.
- [x] T4 (Verify): re-run signer zeroization, preflight policy/lane, and docs-contract checks for story closure.

## Planned Verification Commands

- `cargo test -p kamn-node signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer -- --exact`
- `cargo test -p kamn-node signer::tests::unit_build_kolme_live_managed_signing_key_zeroizes_transient_key_material -- --exact`
- `cargo test -p kamn-node signer::tests::regression_strict_signer_secret_source_precedence_rejects_dual_private_key_envs -- --exact`
- `bash scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh`
- `bash scripts/kolme/test_run_local_kolme_live_deployment_preflight_lane.sh`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs`
