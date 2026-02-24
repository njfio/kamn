# Plan: Issue #5915 - Epic: R65 Security Runtime Remediation and Production Readiness

- Issue: #5915
- Spec: `specs/5915/spec.md`
- Status: Draft
- Last Updated: 2026-02-24

## Approach
1. RED: Add/extend failing tests for conformance cases defined in specs/5915/spec.md.
2. Implement: Replace non-production crypto/signature/hash primitives in production paths.
3. Implement: Deliver real end-to-end message delivery with durable persistence and bounded replay protection.
4. Implement: Harden SDK/service transport and managed-signer execution surfaces.
5. Implement: Expand integration/fuzz/mutation/coverage quality gates.
6. Implement: Reduce architecture/governance drag (core decomposition, script-surface and duplication reduction).
7. REGRESSION: Execute targeted + scoped suite for touched modules and close all failing deltas.
8. VERIFY: Run cargo fmt --check, strict clippy, and issue-scoped tests; collect AC evidence for PR.

## Affected Modules (Initial)
- `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- `docs/security/`
- `docs/architecture/`

## Risks + Mitigations
- Risk: Scope expansion across multiple crates can increase merge and verification time.
  - Mitigation: keep PRs task-scoped and verify each AC with targeted tests before running broader suites.
- Risk: Security/runtime changes can regress existing contracts.
  - Mitigation: preserve and extend regression coverage before behavior changes.
- Risk: Cross-issue dependencies can block downstream tasks.
  - Mitigation: execute in dependency order and keep blockers logged in issue comments.

## Interfaces / Contracts
- Primary contract source: `specs/5915/spec.md`.
- Upstream issue contract: GitHub issue #5915.
- Protocol/API/schema changes require explicit documentation updates and linked follow-up issues when out of scope.

## ADR Requirement
- ADR required if this issue introduces a new dependency, protocol/wire-format change, or architecture boundary change.
