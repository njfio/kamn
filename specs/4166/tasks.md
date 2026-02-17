# Tasks — Issue #4166

- [x] T1 (Red): run failing tests proving missing env-secret precedence-path zeroization.
  Evidence:
  - `cargo test -p kamn-node signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer -- --exact` failed before helper zeroization wiring.
  - `cargo test -p kamn-node regression_signer_secret_source_precedence_path_requires_zeroize_markers` failed before signer-source zeroization marker was present.
- [x] T2 (Green): implement zeroization for precedence failure path and transient signer key material.
  Evidence:
  - Added `private_key_hex.zeroize()` in signer-source precedence helper.
  - Added `key_material.zeroize()` in managed signing key constructor.
- [x] T3 (Refactor): keep helper boundaries explicit and maintain deterministic fail-closed reasons.
  Evidence:
  - Precedence checks still fail closed with `signer_secret_source_precedence_violation`.
  - Zeroization responsibilities are explicit in signer/read and signer-adapter layers.
- [x] T4 (Verify): run targeted kamn-node + docs-contract suites and record outputs.
  Evidence:
  - `cargo test -p kamn-node signer::tests::unit_build_kolme_live_managed_signing_key_zeroizes_transient_key_material -- --exact`
  - `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_signer_secret_zeroization_controls -- --exact`
  - `cargo test -p kamn-core --test threat_control_matrix_docs matrix_contains_signer_secret_zeroization_entry_details -- --exact`
  - `cargo fmt --check`
