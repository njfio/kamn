# Issue #3638 Tasks

- Issue: `#3638`
- Status: `Completed`

## Ordered Tasks
- T1 (Red, Conformance): add failing parity matrix/drift-guard tests for signer migration contracts.
- T2 (Green): implement parity matrix test coverage and docs/source drift guards.
- T3 (Docs): update signer lifecycle migration matrix + guard command references.
- T4 (Regression): run scoped signer suites and docs contracts.
- T5 (Follow-on): implement signer size/ownership budget guards (`#3808`).
- T6 (Verify): run:
  - `cargo fmt --check`
  - `cargo test -p kamn-node --test signer_migration_parity_docs_contract -- --nocapture`
  - `cargo test -p kamn-node signer -- --nocapture`
  - `cargo test -p kamn-node main_tests::signer_tests -- --nocapture`
  - `cargo clippy -p kamn-node -- -D warnings`

## Completion Evidence
- signer migration parity matrix and drift guards are explicit, tested, and documented.
- signer suites remain parity-stable after migration completion slices.
