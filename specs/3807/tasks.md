# Issue #3807 Tasks

- Issue: `#3807`
- Status: `InProgress`

## Ordered Tasks
- T1 (Red, Unit/Conformance): add signer-policy reason-taxonomy contract tests and capture initial fail against missing runtime-network taxonomy markers.
- T2 (Green, Docs): add signer-policy reason taxonomy section/marker list to `docs/foundation/runtime-network.md`.
- T3 (Refactor): keep marker list consolidated between source/docs assertions in the contract test.
- T4 (Regression): run scoped contract and signer suites.
- T5 (Verify): run:
  - `cargo fmt --check`
  - `cargo test -p kamn-node --test signer_policy_reason_taxonomy_contract -- --nocapture`
  - `cargo test -p kamn-node signer -- --nocapture`
  - `cargo test -p kamn-node main_tests::signer_tests -- --nocapture`
  - `cargo clippy -p kamn-node -- -D warnings`

## Completion Evidence
- signer-policy reason markers are guarded by deterministic contract tests.
- runtime-network docs include signer policy reason taxonomy markers with explicit version marker.
- scoped signer suites remain green.
