# Issue #3912 Tasks

- Issue: `#3912`
- Status: `Completed`

## Ordered Tasks
- T1 (Red, Regression): add failing decode-failure redaction test for signer private-key parse path.
- T2 (Green): implement/adjust signer decode path and error assertions to keep key material redacted.
- T3 (Docs): document decode-path zeroization guarantees in runtime-commit architecture doc.
- T4 (Conformance): add source/docs drift guard contract test.
- T5 (Verify): run:
  - `cargo fmt --check`
  - `cargo test -p kamn-node --test signer_secret_hygiene_contract -- --nocapture`
  - `cargo test -p kamn-node signer -- --nocapture`
  - `cargo test -p kamn-node main_tests::signer_tests -- --nocapture`
  - `cargo clippy -p kamn-node -- -D warnings`

## Completion Evidence
- decode-path zeroization and redaction guarantees are test-enforced and documented.
- signer regression suites remain green.
