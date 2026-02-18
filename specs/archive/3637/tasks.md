# Issue #3637 Tasks

- Issue: `#3637`
- Status: `Completed`

## Ordered Tasks
- T1 (Red, Unit/Functional): keep or add failing coverage for nonce retry determinism and managed backend provenance/reason-code behavior before extraction edits.
- T2 (Green, Refactor): extract managed backend control into `signer/managed_backend.rs` and wire `signer.rs` re-exports.
- T3 (Green, Refactor): extract nonce fetch/retry path into `signer/nonce.rs` and wire `signer.rs` re-exports.
- T4 (Regression): run scoped signer tests and fix any behavior drift.
- T5 (Docs): update signer ownership boundaries in `docs/foundation/runtime-network.md`.
- T6 (Verify): run:
  - `cargo fmt --check`
  - `cargo test -p kamn-node signer -- --nocapture`
  - `cargo test -p kamn-node main_tests::signer_tests -- --nocapture`

## Completion Evidence
- `signer.rs` no longer owns managed backend and nonce retry implementation details.
- Scoped signer tests pass with deterministic reason-code behavior preserved.
