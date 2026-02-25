# Plan: Issue #5924 - Task: Replace kamn-core wipe_bytes loop with compiler-safe zeroization

- Issue: #5924
- Spec: `specs/5924/spec.md`
- Status: Implemented
- Last Updated: 2026-02-24

## Approach
1. RED: added regression contracts around `wipe_bytes` zeroization semantics and secret-safe failure output.
2. Implemented `zeroize`-backed erasure in `crates/kamn-core/src/signature_profile.rs` via `bytes.zeroize()`.
3. Added integration round-trip regression to confirm service-auth sign/verify behavior stability after refactor.
4. Verified with targeted tests, `cargo fmt --check`, and strict `kamn-core` clippy.

## Affected Modules
- `crates/kamn-core/src/signature_profile.rs`
- `crates/kamn-core/tests/signature_profile_zeroization_contract.rs`

## Risks + Mitigations
- Risk: Scope expansion across multiple crates can increase merge and verification time.
  - Mitigation: keep PRs task-scoped and verify each AC with targeted tests before running broader suites.
- Risk: Security/runtime changes can regress existing contracts.
  - Mitigation: preserve and extend regression coverage before behavior changes.
- Risk: Cross-issue dependencies can block downstream tasks.
  - Mitigation: execute in dependency order and keep blockers logged in issue comments.

## Interfaces / Contracts
- Primary contract source: `specs/5924/spec.md`.
- Upstream issue contract: GitHub issue #5924.
- Protocol/API/schema changes require explicit documentation updates and linked follow-up issues when out of scope.

## ADR Requirement
- Not required (no dependency or protocol/wire-format change).
