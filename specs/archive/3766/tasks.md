# Issue #3766 Tasks

- Issue: `#3766`
- Status: `Completed`

## Ordered Tasks
- T1 (Red, Functional/Conformance): add failing signer migration docs parity contract test for missing matrix markers.
- T2 (Green): add signer profile/key-source migration parity matrix functional test.
- T3 (Green, Docs): update signer lifecycle docs with migration matrix markers and drift-guard command reference.
- T4 (Regression): run scoped signer and docs contract suites.
- T5 (Verify): run:
  - `cargo fmt --check`
  - `cargo test -p kamn-node --test signer_migration_parity_docs_contract -- --nocapture`
  - `cargo test -p kamn-node main_tests::signer_tests::functional_signer_migration_profile_key_source_parity_matrix -- --exact --nocapture`
  - `cargo test -p kamn-node signer -- --nocapture`
  - `cargo test -p kamn-node main_tests::signer_tests -- --nocapture`

## Completion Evidence
- signer migration parity matrix is explicitly tested.
- signer-lifecycle docs contain required matrix markers and guard commands.
- signer suites remain green.
