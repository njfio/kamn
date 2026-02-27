# Tasks: Issue 6216 - Publish Channel/Task/Bridge Lifecycle Events via WebSocket Fanout

- Issue: #6216
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add websocket fanout regression covering channel/task/bridge lifecycle event publication and sequence progression.
- [x] T2 (GREEN): add new websocket lifecycle event payload structs and publish helpers in `websocket.rs`.
- [x] T3 (GREEN): wire middleware success paths to publish channel/task/bridge lifecycle events.
- [x] T4 (VERIFY): run `cargo fmt --check`, `cargo clippy -p kamn-node -- -D warnings`, and `cargo test -p kamn-node service_api_endpoint::websocket -- --nocapture`.
