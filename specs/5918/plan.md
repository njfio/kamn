# Plan: Issue #5918 - Story: SDK and Service Transport Security Hardening

- Issue: #5918
- Spec: `specs/5918/spec.md`
- Status: Draft
- Last Updated: 2026-02-24

## Approach
1. RED: Add/extend failing tests for conformance cases defined in specs/5918/spec.md.
2. Implement: Harden request construction, HTTPS support, probe validation, pooling, and managed signer execution safety.
3. REGRESSION: Execute targeted + scoped suite for touched modules and close all failing deltas.
4. VERIFY: Run cargo fmt --check, strict clippy, and issue-scoped tests; collect AC evidence for PR.

## Affected Modules (Initial)
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/src/live.rs`
- `crates/kamn-core/src/signer_backend.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`

## Risks + Mitigations
- Risk: Scope expansion across multiple crates can increase merge and verification time.
  - Mitigation: keep PRs task-scoped and verify each AC with targeted tests before running broader suites.
- Risk: Security/runtime changes can regress existing contracts.
  - Mitigation: preserve and extend regression coverage before behavior changes.
- Risk: Cross-issue dependencies can block downstream tasks.
  - Mitigation: execute in dependency order and keep blockers logged in issue comments.

## Interfaces / Contracts
- Primary contract source: `specs/5918/spec.md`.
- Upstream issue contract: GitHub issue #5918.
- Protocol/API/schema changes require explicit documentation updates and linked follow-up issues when out of scope.

## ADR Requirement
- ADR required if this issue introduces a new dependency, protocol/wire-format change, or architecture boundary change.
