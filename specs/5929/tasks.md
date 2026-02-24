# Tasks: Issue #5929 - Task: Harden SDK HTTP request construction against path/header injection

- Issue: #5929
- Spec: `specs/5929/spec.md`
- Plan: `specs/5929/plan.md`
- Status: Implemented
- Last Updated: 2026-02-24

## Ordered Tasks
- T1 (RED / Conformance): add CRLF route/header injection regression tests in SDK service-client integration suite.
- T2 (GREEN / Implementation): enforce route-segment sanitization + header-value validation in SDK request builder.
- T3 (Refactor): centralize auth-header rendering and shared HTTP request validation helpers.
- T4 (Regression): run SDK service-client contract suite and replay existing signed-route/websocket flows.
- T5 (Verify): run `cargo fmt --check` and strict `kamn-sdk` clippy.
- T6 (Process): set spec/plan/tasks status to Implemented and open issue PR evidence.
