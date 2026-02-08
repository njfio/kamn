# Python SDK Beta Slice (Issues #134, #135, #483)

This document captures the first Python SDK implementation slice for MVP workflow parity.

## Scope Delivered
- Added dependency-light Python SDK module: `kamn_sdk.py`.
- Added in-memory `KAMNClient` implementation with:
  - `register`, `resolve`
  - `send`, `receive`, `receive_stream` (async generator)
  - `create_task`, `accept_task`
  - `create_escrow`, `release_escrow`, `balance`
  - `search_agents`, `get_reputation`
- Added Python test coverage: `tests/python/test_sdk.py`.

## Deterministic Behavior
- DIDs, message IDs, task IDs, and escrow IDs use deterministic sequence generators.
- Inbox receives are drain-based and idempotent on repeated reads.
- Async `receive_stream(...)` yields drained messages in deterministic FIFO order.
- Escrow release is one-time and guarded against duplicate release.
- Unknown DID/task/escrow access returns explicit `SDKError`.

## Local Validation
Run from repository root:

```bash
python3 -m unittest tests/python/test_sdk.py
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
