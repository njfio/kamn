# Plan: Issue #5926 - Task: Wire real end-to-end message delivery from /v1/messages/send to recipient state

- Issue: #5926
- Spec: `specs/5926/spec.md`
- Status: Draft
- Last Updated: 2026-02-24

## Approach
1. RED: Add/extend failing tests for conformance cases defined in specs/5926/spec.md.
2. Implement: Route API send requests through runtime queue/transport and persist lifecycle states through delivery.
3. REGRESSION: Execute targeted + scoped suite for touched modules and close all failing deltas.
4. VERIFY: Run cargo fmt --check, strict clippy, and issue-scoped tests; collect AC evidence for PR.

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
