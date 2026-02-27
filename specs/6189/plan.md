# Plan: Issue 6189 - Atomic Service API State Writes

- Issue: #6189
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Introduce a private helper in `message_store.rs` that:
   - creates a temporary file in the destination directory,
   - writes payload bytes,
   - flushes + `sync_all` on temp file,
   - atomically renames temp file to destination path,
   - best-effort `sync_all` on parent directory,
   - cleans up temp file on failures.
2. Switch `ServiceApiMessageStore::persist` to call the helper.
3. Add targeted unit tests for success/failure behavior.
4. Run focused `kamn-node` tests touching persistence.

## Affected Modules

- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint/tests.rs` (if integration assertions are needed)

## Risks and Mitigations

1. Cross-platform rename semantics:
   - Mitigation: create temp file in same directory and use `fs::rename`.
2. Temporary-file leakage on errors:
   - Mitigation: explicit cleanup paths + regression tests.
3. Behavior drift for existing endpoints:
   - Mitigation: run existing persistence tests unchanged.

## Contracts / Interfaces

No public API changes.
Internal persistence implementation contract changes from direct write to atomic replace.
