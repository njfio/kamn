# Issue #3917 Tasks

- Issue: `#3917`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add docs-contract parity tests for signer secret-lifecycle markers and closure chain declarations.
- T2 (Green): add required marker sections to CI strategy and production next-steps docs.
- T3 (Verify): run scoped docs-contract and signer regression suites.

## Verification Commands
- `cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract docs_declare_signer_secret_lifecycle_policy_markers_and_closure_chain -- --exact --nocapture`
- `cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract -- --nocapture`

## Completion Evidence
- docs marker parity fails closed on missing signer secret-lifecycle policy declarations.
