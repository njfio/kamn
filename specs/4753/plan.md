# Plan: Issue #4753

Status: Reviewed
Issue: #4753

## Approach

1. Add explicit `ZkDesignError` invariant variant for rank-list-empty condition and return it via
   `ok_or(...)` in phase-4 recommendation flow.
2. Replace notifications consumer `expect()` with explicit `Option` handling that preserves
   reconnect budget + deterministic unavailable reason behavior.
3. Run targeted zk + notifications selectors and touched-crate lint checks.

## Affected Modules

- `crates/kamn-core/src/zk_message_proofs.rs`
- `crates/kamn-core/src/kolme_runtime_commit/notifications_consumer.rs`

## Risks / Mitigations

- Risk: new error variant could alter caller expectations.
  - Mitigation: keep existing `Result<_, ZkDesignError>` signature unchanged and only widen enum
    with a deterministic invariant-specific variant.

- Risk: notifications connection handling change could affect reconnect behavior.
  - Mitigation: reuse existing reconnect attempt increment + exhausted reason contract and keep read
    failure handling unchanged.

## Interface Contract

- No public API signature changes.
- No wire/protocol format changes.
- Error handling semantics become stricter fail-closed (panic removed, typed error returned).

## Review Note (P1)

- This is a P1 multi-module change; PR explicitly requests human review before merge.
