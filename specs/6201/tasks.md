# Tasks: Issue 6201 - Reduce E2E Driver Duplication Surface

- Issue: #6201
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add shared-helper unit tests for live-scenario gating, replay marker validation, and percentile behavior.
- [x] T2 (GREEN): introduce `drivers/shared.rs` and implement extracted helper functions.
- [x] T3 (GREEN): update `sdk_direct.rs`, `cli_scripted.rs`, and `mcp_agent.rs` to remove local duplicate helpers.
- [x] T4 (REGRESSION): run `cargo test -p kamn-e2e-harness`.
- [x] T5 (VERIFY): run `cargo fmt --check` and `cargo clippy -p kamn-e2e-harness -- -D warnings`.
