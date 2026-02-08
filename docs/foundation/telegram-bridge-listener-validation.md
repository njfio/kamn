# Telegram Bridge Listener-Validated Inbound Flow (Issues #222, #223)

This document captures the first implementation slice for Telegram bridge inbound processing with listener validation.

## Scope Delivered
- Added `crates/kamn-core/src/telegram_bridge.rs` with:
  - `TelegramBridgeConfig` for bridge agent DID, authorized listener DIDs, and channel route map.
  - `TelegramInboundRequest` as inbound request envelope with listener DID context.
  - `TelegramBridgeEngine` wrapper over generic bridge adapter flow.
  - typed errors via `TelegramBridgeError`.
- Added integration tests in `crates/kamn-core/tests/telegram_bridge.rs`.

## Validation Rules
- Bridge agent DID and listener DIDs must parse as `kamn:did:agent:*`.
- At least one authorized listener DID is required.
- At least one Telegram channel route is required.
- Each route target DID must be valid.
- Inbound processing requires:
  - listener DID must be authorized.
  - inbound `external_channel_id` must exist in route map.
  - inbound `target_agent_did` must match mapped route target.

## Processing Flow
- `process_inbound(...)`:
  - validates listener + route constraints.
  - delegates canonical normalization to `BridgeAdapterEngine` with Telegram platform adapter.
- `process_inbound_to_envelope(...)`:
  - runs listener/route validation.
  - converts inbound message into canonical KAMN envelope.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test telegram_bridge
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
