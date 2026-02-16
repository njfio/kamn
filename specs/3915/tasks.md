# Issue #3915 Tasks

- Issue: `#3915`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add failing secret-lifecycle policy/docs parity contract tests.
- T2 (Green): implement policy checker helper coverage for fallback-secret and missing-marker fail-closed behavior.
- T3 (Docs): add CI and next-steps closure markers for signer secret-lifecycle policy.
- T4 (Regression): run signer and policy contract suites.
- T5 (Verify): run:
  - `cargo fmt --check`
  - `cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract -- --nocapture`
  - `cargo test -p kamn-node signer -- --nocapture`
  - `cargo test -p kamn-node main_tests::signer_tests -- --nocapture`
  - `cargo clippy -p kamn-node -- -D warnings`

## Completion Evidence
- secret-lifecycle policy and docs parity checks are deterministic and fail-closed.
