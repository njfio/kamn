# Tasks - Issue #4158

- [x] T1 (Red): define failing signer zeroization, explicit-material, and rotation preflight governance checks (delivered in child stories).
- [x] T2 (Green): complete signer security and rotation readiness implementations (`#4159`, `#4160`).
- [x] T3 (Refactor/Docs): align ops/deploy/release docs contracts with deterministic governance markers.
- [x] T4 (Verify): re-run representative cross-story checks for epic closure.

## Planned Verification Commands

- `cargo test -p kamn-node signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer -- --exact`
- `cargo test -p kamn-node signer::tests::unit_build_kolme_live_managed_signing_key_zeroizes_transient_key_material -- --exact`
- `bash scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh`
- `bash scripts/kolme/test_run_local_kolme_live_deployment_preflight_lane.sh`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs`
- `cargo test -p kamn-core --test kolme_devnet_ops_docs`
- `cargo test -p kamn-core --test release_gonogo_checklist_docs`
