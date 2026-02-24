# Plan: Issue #5936 - Task: Wire Data Layer M0-M11 into service runtime paths with staged activation

- Issue: #5936
- Spec: `specs/5936/spec.md`
- Status: Draft
- Last Updated: 2026-02-24

## Approach
1. RED: Add/extend failing tests for conformance cases defined in specs/5936/spec.md.
2. Implement: Define staged integration points and connect modules to real API/runtime workflows.
3. REGRESSION: Execute targeted + scoped suite for touched modules and close all failing deltas.
4. VERIFY: Run cargo fmt --check, strict clippy, and issue-scoped tests; collect AC evidence for PR.

## Affected Modules (Initial)
- `crates/kamn-core/src/data_layer_m0.rs` to `crates/kamn-core/src/data_layer_m11.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/main.rs`

## Risks + Mitigations
- Risk: Scope expansion across multiple crates can increase merge and verification time.
  - Mitigation: keep PRs task-scoped and verify each AC with targeted tests before running broader suites.
- Risk: Security/runtime changes can regress existing contracts.
  - Mitigation: preserve and extend regression coverage before behavior changes.
- Risk: Cross-issue dependencies can block downstream tasks.
  - Mitigation: execute in dependency order and keep blockers logged in issue comments.

## Interfaces / Contracts
- Primary contract source: `specs/5936/spec.md`.
- Upstream issue contract: GitHub issue #5936.
- Protocol/API/schema changes require explicit documentation updates and linked follow-up issues when out of scope.

## ADR Requirement
- ADR required if this issue introduces a new dependency, protocol/wire-format change, or architecture boundary change.
