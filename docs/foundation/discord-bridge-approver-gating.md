# Discord Bridge Approver-Gated Outbound Flow (Issues #220, #221)

This document captures the first implementation slice for Discord bridge processing with approver quorum gating on outbound actions.

## Scope Delivered
- Added `crates/kamn-core/src/discord_bridge.rs` with:
  - `DiscordBridgeConfig` for bridge DID, listener allowlist, approver allowlist, required approval threshold, and route map.
  - `DiscordInboundRequest` and `DiscordBridgeEngine` for listener-validated inbound normalization.
  - `process_outbound_with_approvals(...)` for approver-gated outbound dispatch.
  - `DiscordOutboundApproval` and `DiscordOutboundDispatch` to preserve deterministic approval metadata for auditing.
  - typed error handling via `DiscordBridgeError`.
- Added integration tests in `crates/kamn-core/tests/discord_bridge.rs`.

## Validation Rules
- Bridge/listener/approver DIDs must parse as `kamn:did:agent:*`.
- At least one authorized listener and approver DID is required.
- `required_approvals` must be between `1` and approver allowlist size.
- At least one route entry is required and route target DIDs must be valid.
- Outbound processing requires:
  - destination channel mapped in route map.
  - approver set contains no duplicates.
  - all approvers are authorized.
  - provided approvals satisfy quorum threshold.

## Processing Flow
- `process_inbound(...)`:
  - validates listener identity and route target mapping.
  - normalizes inbound payload through `BridgeAdapterEngine` with Discord platform adapter.
- `process_inbound_to_envelope(...)`:
  - preserves listener/route checks.
  - projects to canonical KAMN message envelope.
- `process_outbound_with_approvals(...)`:
  - enforces channel routing + approver quorum checks.
  - delegates outbound translation to `BridgeAdapterEngine`.
  - returns translated payload plus approval metadata for audit trails.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test discord_bridge
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
