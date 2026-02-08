# Secure Off-Chain Retrieval, Cache, and Access Controls (Issues #164 / #165)

This document captures the first implementation slice for secure retrieval paths with authorization checks, cache TTL handling, and auditability.

## Scope Delivered
- Added `crates/kamn-core/src/content_retrieval.rs` with:
  - `ContentRetrievalConfig` for bounded cache TTL.
  - `ContentRetrievalRequest` and `ContentRetrievalScope` for task/channel-scoped retrieval context.
  - `ContentRetrievalEngine` with authorization, cache, and retrieval workflow.
  - `ContentRetrievalAuditEvent` and `ContentRetrievalOutcome` for deterministic audit signals.
  - typed errors via `ContentRetrievalError`.
- Added integration and regression tests in `crates/kamn-core/tests/content_retrieval_access_cache.rs`.

## Authorization and Cache Rules
- Task scope:
  - explicit allowlist via `grant_task_read(task_id, requester)`.
  - unauthorized task reads return `ContentRetrievalError::Unauthorized`.
- Channel scope:
  - retrieval requires `ChannelPermissionEngine` and `ChannelAction::Read` authorization.
- Cache behavior:
  - cache key binds `requester + scope + cid`.
  - authorization checks always run before cache access.
  - TTL expiry forces re-fetch from storage and cache refresh.

## Audit and Security Guarantees
- Each retrieval attempt appends an audit event:
  - outcome `Allowed` or `Denied`
  - scope, requester, CID, timestamp
  - cache-hit marker
- Storage integrity checks are enforced (`verify`) before payload retrieval.
- Cache entries cannot be reused across different requester/scope combinations.

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test content_retrieval_access_cache --test content_retrieval_access_cache_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```
