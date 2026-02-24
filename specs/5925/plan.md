# Plan: Issue #5925 - Task: Replace FNV-based key lifecycle audit chain hashing with cryptographic hash

- Issue: #5925
- Spec: `specs/5925/spec.md`
- Status: Draft
- Last Updated: 2026-02-24

## Approach
1. RED: Add/extend failing tests for conformance cases defined in specs/5925/spec.md.
2. Implement: Replace audit-chain hash with SHA-256/BLAKE3 and version marker migration.
3. REGRESSION: Execute targeted + scoped suite for touched modules and close all failing deltas.
4. VERIFY: Run cargo fmt --check, strict clippy, and issue-scoped tests; collect AC evidence for PR.

## Affected Modules (Initial)
- `crates/kamn-core/src/key_lifecycle.rs`
- `crates/kamn-core/src/audit_exports.rs`
- `crates/kamn-core/src/task_artifacts.rs`
- `crates/kamn-core/tests/`

## Risks + Mitigations
- Risk: Scope expansion across multiple crates can increase merge and verification time.
  - Mitigation: keep PRs task-scoped and verify each AC with targeted tests before running broader suites.
- Risk: Security/runtime changes can regress existing contracts.
  - Mitigation: preserve and extend regression coverage before behavior changes.
- Risk: Cross-issue dependencies can block downstream tasks.
  - Mitigation: execute in dependency order and keep blockers logged in issue comments.

## Interfaces / Contracts
- Primary contract source: `specs/5925/spec.md`.
- Upstream issue contract: GitHub issue #5925.
- Protocol/API/schema changes require explicit documentation updates and linked follow-up issues when out of scope.

## ADR Requirement
- ADR required if this issue introduces a new dependency, protocol/wire-format change, or architecture boundary change.
