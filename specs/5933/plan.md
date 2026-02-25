# Plan: Issue #5933 - Task: Decompose kamn-core into focused crates with phase-1 extraction

- Issue: #5933
- Spec: `specs/5933/spec.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Approach
1. RED: Add/extend failing tests for conformance cases defined in specs/5933/spec.md.
2. Implement: Extract first tranche into focused crates with stable boundaries and minimal API breakage.
3. REGRESSION: Execute targeted + scoped suite for touched modules and close all failing deltas.
4. VERIFY: Run cargo fmt --check, strict clippy, and issue-scoped tests; collect AC evidence for PR.

## Affected Modules (Initial)
- `Cargo.toml`
- `crates/kamn-runtime-guards/Cargo.toml`
- `crates/kamn-runtime-guards/src/lib.rs`
- `crates/kamn-runtime-guards/src/anti_spam.rs`
- `crates/kamn-runtime-guards/src/fairness_policy.rs`
- `crates/kamn-runtime-guards/src/quota_policy.rs`
- `crates/kamn-runtime-guards/src/message_delivery_guards.rs`
- `crates/kamn-runtime-guards/src/retention_engine.rs`
- `crates/kamn-runtime-guards/src/watchdog.rs`
- `crates/kamn-core/Cargo.toml`
- `crates/kamn-core/src/anti_spam.rs`
- `crates/kamn-core/src/fairness_policy.rs`
- `crates/kamn-core/src/quota_policy.rs`
- `crates/kamn-core/src/message_delivery_guards.rs`
- `crates/kamn-core/src/retention_engine.rs`
- `crates/kamn-core/src/watchdog.rs`
- `crates/kamn-core/tests/issue_5933_runtime_guards_extraction.rs`
- `docs/architecture/kamn-core-module-map.md`
- `docs/architecture/README.md`
- `docs/architecture/adr-002-runtime-guards-phase1-extraction.md`

## Risks + Mitigations
- Risk: Scope expansion across multiple crates can increase merge and verification time.
  - Mitigation: keep PRs task-scoped and verify each AC with targeted tests before running broader suites.
- Risk: Security/runtime changes can regress existing contracts.
  - Mitigation: preserve and extend regression coverage before behavior changes.
- Risk: Cross-issue dependencies can block downstream tasks.
  - Mitigation: execute in dependency order and keep blockers logged in issue comments.

## Interfaces / Contracts
- Primary contract source: `specs/5933/spec.md`.
- Upstream issue contract: GitHub issue #5933.
- Protocol/API/schema changes require explicit documentation updates and linked follow-up issues when out of scope.

## ADR Requirement
- ADR required if this issue introduces a new dependency, protocol/wire-format change, or architecture boundary change.

## Execution Evidence
1. RED:
   - `cargo test -p kamn-core --test issue_5933_runtime_guards_extraction`
   - failed with unresolved crate `kamn_runtime_guards`.
2. GREEN:
   - created `kamn-runtime-guards` crate and compatibility shims in `kamn-core`.
   - `cargo test -p kamn-core --test issue_5933_runtime_guards_extraction` passed.
3. REGRESSION / VERIFY:
   - `cargo test -p kamn-runtime-guards` passed.
   - `cargo fmt --check` passed.
   - `cargo clippy -p kamn-runtime-guards --all-targets -- -D warnings` passed.
   - `cargo clippy -p kamn-core --tests -- -D warnings` passed.
