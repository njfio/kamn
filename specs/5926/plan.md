# Plan: Issue #5926 - Task: Wire real end-to-end message delivery from /v1/messages/send to recipient state

- Issue: #5926
- Spec: `specs/5926/spec.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Approach
1. RED: Add/extend failing tests for conformance cases defined in specs/5926/spec.md.
2. Implement: Route API send requests through runtime queue/transport and persist lifecycle states through delivery.
3. REGRESSION: Execute targeted + scoped suite for touched modules and close all failing deltas.
4. VERIFY: Run cargo fmt --check, strict clippy, and issue-scoped tests; collect AC evidence for PR.

## Delivered Implementation
- Existing runtime/service-api implementation on `main` already provides:
  - durable message persistence in `crates/kamn-node/src/service_api_endpoint/message_store.rs`,
  - durable relay spool enqueue/drain + state projection in `crates/kamn-node/src/service_api_endpoint/state_io.rs`,
  - `/v1/messages/send` to relay-spool wiring in `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`,
  - daemon relay tick-loop processing in `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`.
- Added verification-strength assertions in:
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
  - `crates/kamn-node/src/main_tests/runtime_tests/daemon_relay_projection_tests.rs`
  to ensure observability values reflect non-zero processed work for real delivery flows.

## Affected Modules (Initial)
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/main.rs`
- `crates/kamn-core/src/message_lifecycle.rs`
- `crates/kamn-core/src/runtime_service.rs`

## Risks + Mitigations
- Risk: Scope expansion across multiple crates can increase merge and verification time.
  - Mitigation: keep PRs task-scoped and verify each AC with targeted tests before running broader suites.
- Risk: Security/runtime changes can regress existing contracts.
  - Mitigation: preserve and extend regression coverage before behavior changes.
- Risk: Cross-issue dependencies can block downstream tasks.
  - Mitigation: execute in dependency order and keep blockers logged in issue comments.

## Interfaces / Contracts
- Primary contract source: `specs/5926/spec.md`.
- Upstream issue contract: GitHub issue #5926.
- Protocol/API/schema changes require explicit documentation updates and linked follow-up issues when out of scope.

## ADR Requirement
- ADR required if this issue introduces a new dependency, protocol/wire-format change, or architecture boundary change.
