# Authoritative Live Settlement Driver Parity Slice

## Claim

SDK-direct, CLI-scripted, and MCP-agent release surfaces preserve the same
service-issued `authoritative_settlement` object for a bridge-authorized
escrow release. The object binds both receipt digests, actor, task, escrow,
economic terms, transaction finality, receipt-chain commitment, and durable
operation identity.

This is adapter parity for the bounded Solana devnet bridge-authorized path.
It is not generalized settlement and is not production readiness.

## Canonical Flow

1. Finalize one bridge receipt through the live bridge path from #7160.
2. Release the bound escrow with `authority_mode=bridge-receipt`, the bridge
   ID, and one durable idempotency key.
3. Persist the settlement intent and consume the finalized bridge receipt.
4. Return the same authoritative observation on retry and after restart.
5. Normalize SDK-direct, CLI-scripted, and MCP-agent JSON with the shared
   harness validator.

The release path consumes the already-finalized bridge transaction. It does
not submit another settlement transaction.

## Required Evidence

- All three entrypoints expose an identical receipt digest pair.
- All three expose the same transaction signature and finalized slot.
- Actor, task, escrow, recipient, amount, asset, network, and terms match.
- Receipt-chain commitment and idempotency identity match byte-for-byte.
- Missing, partial, tampered, cross-resource, reordered, and replayed
  observations fail closed.
- Retry and restart preserve exactly one transfer and zero additional
  settlement submission attempts.

## Commands

```bash
cargo test -p kamn-e2e-harness --test authoritative_settlement_observation_contract
cargo test -p kamn-e2e-harness --test authoritative_settlement_driver_parity_contract
RUST_TEST_THREADS=1 cargo test -p kamn-e2e-harness --test authoritative_settlement_entrypoint_parity
cargo test -p kamn-node --bin kamn-node integration_bridge_authorized_release_reuses_finalized_bridge_receipt
cargo test -p kamn-sdk
cargo test -p kamn-cli
cargo test -p kamn-mcp-server
```

The external live finality proof remains the #7160 artifact. This slice
composes that proof with deterministic real service release and adapter
contracts; it does not claim a fresh funded transfer by every adapter.

`submit_bridge_message` is intentionally not terminal authority because no
finalized receipt exists yet. `forward_bridge_message` is the finalizing MCP
mutation and is service-authority wrapped.
