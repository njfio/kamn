# Tasks — Issue #4165

- [x] T1 (Red): add failing tests for env-secret buffer zeroization on signer precedence failure.
  Evidence:
  - `cargo test -p kamn-node signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer -- --exact` failed with:
    `strict signer precedence violation must scrub env-secret private key buffers`
  - `cargo test -p kamn-node regression_signer_secret_source_precedence_path_requires_zeroize_markers` failed with:
    `signer source precedence helper must explicitly zeroize env-secret buffers`
- [x] T2 (Green): wire/adjust signer precedence secret handling so new tests pass.
  Evidence:
  - Added explicit zeroization in `ensure_kolme_live_strict_signer_secret_source_precedence_and_zeroize`.
  - Added signer-source regression marker checks in `main_tests/signer_tests.rs`.
- [x] T3 (Refactor): keep zeroization checks explicit and maintain deterministic reason-code behavior.
  Evidence:
  - Kept strict precedence reason marker unchanged: `signer_secret_source_precedence_violation`.
  - Centralized precedence-path scrubbing in a dedicated helper.
- [x] T4 (Verify): run targeted unit/functional/regression tests and capture evidence.
  Evidence:
  - `cargo test -p kamn-node signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer -- --exact`
  - `cargo test -p kamn-node regression_signer_secret_source_precedence_path_requires_zeroize_markers`
  - `cargo test -p kamn-node signer::tests::regression_strict_signer_secret_source_precedence_rejects_dual_private_key_envs -- --exact`
  - `cargo fmt --check`
