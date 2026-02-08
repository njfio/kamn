# Role Smoke Scenarios (Issue #18)

This document describes baseline smoke scenarios for processor/listener/approver behavior.

## Scope
- Validate transaction intake at the processor boundary.
- Validate block production from pending processor transactions.
- Validate gossip propagation expectations to listener and approver roles.

## Model
The smoke harness is intentionally lightweight and deterministic:
- `RoleSmokeNetwork` simulates one processor, one listener, and one approver.
- `submit_transaction(...)` validates ID, nonce, and payload before ingest.
- `produce_block(...)` orders transactions deterministically by nonce then ID.
- Gossip behavior is explicit via the `gossip_enabled` toggle.

## Failure Modes
- Duplicate transaction IDs are rejected.
- Non-positive nonces are rejected.
- Empty payloads are rejected.
- Producing a block with an empty processor mempool fails explicitly.

## Validation Commands
Run from repository root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```
