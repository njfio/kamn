# Spec: Issue #4753

Status: Reviewed
Issue: #4753
Parent: #4399
Milestone: specs/milestones/r27-36-deep-validation-hardening-concurrency-safety-and-observability-emission-governance/index.md
Priority: P1

## Problem Statement

`docs/security/secure-coding.md` defines production `.expect(` usage as a violation marker, but two
production paths in `kamn-core` still used `expect()` and could panic under invariant drift.

## Scope

In scope:
- Replace `expect()` in zk option-ranking recommendation flow with typed error return.
- Replace `expect()` in notifications connection-read flow with deterministic fail-closed `Result`
  handling.
- Keep existing behavior and retry/budget contracts stable.

Out of scope:
- Repo-wide production panic-path cleanup.
- Protocol/wire-format changes.

## Acceptance Criteria

AC-1:
Given a non-empty option set in zk phase-4 planning, when ranking cannot produce the first item,
then the function returns a typed `ZkDesignError` instead of panicking.

AC-2:
Given notifications consumer read flow, when a connection is unexpectedly absent before read, then
the consumer fails closed using existing reconnect budget semantics instead of panicking.

AC-3:
Given existing zk and notifications conformance selectors, when targeted test selectors run, then
all pass with no behavior regressions.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `cargo test -p kamn-core --test zk_message_proofs`
  - Expectation: zk proof selectors pass and recommendation flow has no production `expect()` panic
    path.

- C-02 (AC-2, Functional/Integration):
  - Test: `cargo test -p kamn-core --test kolme_runtime_commit_notifications`
  - Expectation: notifications reconnect/decode/integration selectors pass and no production
    connection-read `expect()` remains.

- C-03 (AC-3, Contract/Regression):
  - Test: `cargo fmt --check && cargo clippy -p kamn-core -- -D warnings`
  - Expectation: touched modules compile/lint clean with deterministic fail-closed handling.

## Success Metrics / Observable Signals

- `crates/kamn-core/src/zk_message_proofs.rs` contains no production ranking-path `expect()`.
- `crates/kamn-core/src/kolme_runtime_commit/notifications_consumer.rs` contains no production
  connection-read `expect()`.
- Targeted zk + notifications selectors remain green.
