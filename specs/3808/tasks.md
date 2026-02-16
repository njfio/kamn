# Issue #3808 Tasks

- Issue: `#3808`
- Status: `InProgress`

## Ordered Tasks
- T1 (Red, Functional/Conformance): add failing signer extraction budget/docs policy contract checks.
- T2 (Green): implement signer extraction budget + ownership marker checks.
- T3 (Green, Docs): add CI strategy signer extraction budget guard policy markers and command.
- T4 (Regression): run signer budget contract and scoped signer suites.
- T5 (Verify): run:
  - `cargo fmt --check`
  - `cargo test -p kamn-node --test signer_extraction_budget_contract -- --nocapture`
  - `cargo test -p kamn-node signer -- --nocapture`
  - `cargo test -p kamn-node main_tests::signer_tests -- --nocapture`
  - `cargo clippy -p kamn-node -- -D warnings`

## Completion Evidence
- signer monolith budget regrowth fails closed in contract test.
- signer ownership markers and CI guard docs stay enforced.
- signer suites remain green.
