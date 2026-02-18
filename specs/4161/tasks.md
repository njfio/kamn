# Tasks - Issue #4161

- [x] T1 (Red): add failing-first precedence-path zeroization regression coverage (`#4165`).
- [x] T2 (Green): implement decoded/transient key buffer zeroization (`#4166`).
- [x] T3 (Refactor/Docs): ensure docs-contract markers cover zeroization controls.
- [x] T4 (Verify): re-run signer zeroization regression tests and docs-contract checks for parent closure.

## Planned Verification Commands

- `cargo test -p kamn-node signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer -- --exact`
- `cargo test -p kamn-node signer::tests::unit_build_kolme_live_managed_signing_key_zeroizes_transient_key_material -- --exact`
- `cargo test -p kamn-node regression_signer_secret_source_precedence_path_requires_zeroize_markers`
- `cargo test -p kamn-node signer::tests::regression_strict_signer_secret_source_precedence_rejects_dual_private_key_envs -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_signer_secret_zeroization_controls -- --exact`
- `cargo test -p kamn-core --test threat_control_matrix_docs matrix_contains_signer_secret_zeroization_entry_details -- --exact`
