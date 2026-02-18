# Issue #3914 Tasks

- Issue: `#3914`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add failing regression test for decode-failure redaction.
- T2 (Green): enforce/update decode-failure behavior to keep raw key input redacted.
- T3 (Conformance): add source/docs secret-hygiene contract checks.
- T4 (Docs): update CI strategy and runtime-commit docs markers for redaction guard policy.
- T5 (Verify): run scoped signer and contract suites.

## Verification Commands
- `cargo fmt --check`
- `cargo test -p kamn-node signer::tests::regression_signer_private_key_decode_failure_redacts_sensitive_input -- --exact --nocapture`
- `cargo test -p kamn-node --test signer_secret_hygiene_contract -- --nocapture`
- `cargo test -p kamn-node signer -- --nocapture`
- `cargo test -p kamn-node main_tests::signer_tests -- --nocapture`

## Completion Evidence
- secret-redaction regression checks stay active and pass.
