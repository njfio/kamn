# Issue #4320 Tasks

- Issue: `#4320`
- Status: `InProgress`

## Ordered Tasks
- T1 (Red): add peer adapter reason projection + multi-process hook contract tests.
- T2 (Green): implement deterministic reason projection and hook APIs in `p2p_transport`.
- T3 (Docs): update release go/no-go checklist with peer reason taxonomy and multi-process validation references.
- T4 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`

## Completion Evidence
- Deterministic projection and hook contracts are test-covered across required categories.
- Release go/no-go checklist includes peer reason taxonomy references enforced by docs tests.
