# Issue #3811 Tasks

- Issue: `#3811`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add failing signer adapter boundary/docs contract tests.
- T2 (Green): extract adapter-owned symbols to `signer_adapter` module and re-export from `signer.rs`.
- T3 (Docs): add signer adapter boundary markers/guard command in architecture docs.
- T4 (Regression): run signer contract and scoped signer suite commands.
- T5 (Verify): run:
  - `cargo fmt --check`
  - `cargo test -p kamn-node --test signer_adapter_boundary_contract -- --nocapture`
  - `cargo test -p kamn-node signer -- --nocapture`
  - `cargo test -p kamn-node main_tests::signer_tests -- --nocapture`
  - `cargo clippy -p kamn-node -- -D warnings`

## Completion Evidence
- signer adapter boundary/re-export drift checks fail closed and signer suite parity remains stable.
