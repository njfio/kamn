# Cross-Chain Bridge Adapters (Ethereum and Solana) (Issues #232, #233)

This document captures the first implementation slice for cross-chain bridge adapters covering Ethereum and Solana inbound/outbound pathways.

## Scope Delivered
- Added `crates/kamn-core/src/cross_chain_bridge.rs` with:
  - `CrossChainNetwork` enum (`Ethereum`, `Solana`).
  - `CrossChainBridgeConfig` for listener and approver allowlists, approval threshold, and chain route maps.
  - `CrossChainBridgeEngine` that wires chain-specific adapter engines.
  - `process_inbound(...)` and `process_inbound_to_envelope(...)` with listener and route validation.
  - `process_outbound_with_approvals(...)` with approver quorum enforcement and deterministic approval metadata.
  - typed error handling via `CrossChainBridgeError`.
- Added integration tests in `crates/kamn-core/tests/cross_chain_bridge.rs`.

## Validation Rules
- Bridge/listener/approver DIDs must parse as `kamn:did:agent:*`.
- Listener and approver allowlists must be non-empty.
- `required_approvals` must be between `1` and approver allowlist size.
- Ethereum and Solana route maps must both be present and non-empty.
- Inbound processing requires:
  - authorized listener DID.
  - route lookup hit for chain-specific `external_channel_id`.
  - `target_agent_did` equal to mapped route target DID.
- Outbound processing requires:
  - destination channel present in chain-specific route map.
  - authorized approver set with no duplicates.
  - provided approvals satisfy quorum threshold.

## Processing Flow
- `process_inbound(...)`:
  - validates listener and chain route mapping.
  - normalizes payload through chain-specific bridge adapter engine.
- `process_inbound_to_envelope(...)`:
  - reuses inbound validation path.
  - projects inbound message to canonical KAMN envelope.
- `process_outbound_with_approvals(...)`:
  - validates route + approver quorum.
  - translates outbound payload via selected chain adapter engine.
  - returns translated payload and approval metadata for audit trails.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test cross_chain_bridge
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
