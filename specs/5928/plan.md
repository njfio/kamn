# Plan: Issue #5928 - Task: Bound replay guard memory with TTL/capacity eviction

- Issue: #5928
- Spec: `specs/5928/spec.md`
- Status: Implemented
- Last Updated: 2026-02-24

## Approach
1. RED: Added replay-guard capacity and TTL regression tests in `crates/kamn-node/src/service_api_endpoint/auth.rs`.
2. Implemented bounded replay cache (`ServiceApiReplayGuard`) with capacity + TTL eviction policy.
3. Wired middleware auth path to use bounded replay cache instead of unbounded `BTreeSet`.
4. Verified targeted replay regressions + scoped formatting/clippy checks.

## Affected Modules (Initial)
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `crates/kamn-node/src/service_api_endpoint/server.rs`
- `crates/kamn-node/src/main_tests/`

## Risks + Mitigations
- Risk: Scope expansion across multiple crates can increase merge and verification time.
  - Mitigation: keep PRs task-scoped and verify each AC with targeted tests before running broader suites.
- Risk: Security/runtime changes can regress existing contracts.
  - Mitigation: preserve and extend regression coverage before behavior changes.
- Risk: Cross-issue dependencies can block downstream tasks.
  - Mitigation: execute in dependency order and keep blockers logged in issue comments.

## Interfaces / Contracts
- Primary contract source: `specs/5928/spec.md`.
- Upstream issue contract: GitHub issue #5928.
- Protocol/API/schema changes require explicit documentation updates and linked follow-up issues when out of scope.

## ADR Requirement
- Not required (no dependency or protocol/wire-format change).
