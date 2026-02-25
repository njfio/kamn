# Tasks: Issue #5947 - Task: Restore service_api_endpoint root line-budget contract by removing redundant delegates

- Issue: #5947
- Spec: `specs/5947/spec.md`
- Plan: `specs/5947/plan.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Ordered Tasks
- T1 (RED / Conformance): confirmed CI failure context on root budget drift (`935 > 900`).
- T2 (GREEN / Implementation): removed root delegates from `service_api_endpoint.rs` and switched call sites to direct `auth/payload/websocket` paths.
- T3 (Refactor): reduced duplicate forwarding surface and restored module ownership boundaries.
- T4 (Regression): ran extraction contract and targeted route/websocket tests.
- T5 (Verify): ran `cargo fmt --check` and strict `kamn-node` clippy.
- T6 (Process): marked lifecycle artifacts Implemented and prepared PR/issue evidence links.
