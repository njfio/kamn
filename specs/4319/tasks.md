# Issue #4319 Tasks

- Issue: `#4319`
- Status: `InProgress`

## Ordered Tasks
- T1 (Red): add peer-integrity drift + retry-timeout misclassification tests across required categories.
- T2 (Docs): update `docs/planning/kolme-devnet-ops.md` with contract markers for drift and timeout classification.
- T3 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout`
  - `cargo test -p kamn-core --test kolme_devnet_ops_docs`

## Completion Evidence
- New deterministic tests fail if drift/misclassification is accepted.
- Docs include explicit peer integrity drift and timeout classification markers.
- Scoped verification commands pass.
