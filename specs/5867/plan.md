# Plan: Issue #5867 - End-to-End Message Delivery Continuity

- Issue: #5867
- Spec: `specs/5867/spec.md`
- Status: Draft
- Last Updated: 2026-02-24

## Approach
1. Add RED end-to-end lifecycle tests (recipient + non-recipient).
2. Implement minimal lifecycle transition/authorization fixes.
3. Run regression on message lifecycle contracts.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`

## ADR Requirement
- Not required.
