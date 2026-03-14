# Durable Cross-Node Relay Slice

This runbook documents one current KAMN service-API slice on `main` that proves durable sender spool enqueue, fail-closed pending spool preservation, later successful relay projection, and recipient-visible delivered state from durable state after a fresh reader boot.

## Scope Delivered
- sender enqueue to relay spool
- fail-closed pending spool preservation when routing is missing
- later successful relay projection to a live recipient
- recipient-visible delivered state from the durable state file after fresh boot

## Proof Path
This slice is anchored to three current-main tests:
- `integration_runtime_daemon_without_route_map_preserves_relay_spool_entries`
- `integration_service_api_endpoint_cross_node_relay_delivery_contract`
- `integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract`

Related restart/status guard:
- `regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart`

## What This Proves
- sender enqueue to relay spool is durable before daemon forwarding succeeds
- fail-closed pending spool preservation keeps queued relay work when no route map or recipient path is available
- a later successful relay projection drains the relay spool and advances sender state
- recipient-visible delivered state is readable from the durable state file after a fresh reader boot
- the same runtime surface can express both `relayed` and `delivered` states depending on reader identity and phase

## What This Does Not Prove
- production deployment readiness
- consensus or multi-node finality
- escrow settlement or bridge finality
- global delivery guarantees under arbitrary partitions

## Operator Commands
Run the exact proof targets from a clean checkout:

```bash
cargo test -p kamn-node integration_runtime_daemon_without_route_map_preserves_relay_spool_entries -- --exact --nocapture
cargo test -p kamn-node integration_service_api_endpoint_cross_node_relay_delivery_contract -- --exact --nocapture
cargo test -p kamn-node integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract -- --exact --nocapture
cargo test -p kamn-node regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart -- --exact --nocapture
```

## Expected Evidence
- the sender relay spool remains populated during fail-closed pending spool preservation
- the later successful relay projection drains the relay spool
- sender durable state moves through queued/relayed behavior without silent fallback
- recipient query exposes `delivered`
- non-recipient restart checks preserve `relayed` instead of incorrectly promoting delivery

## Runtime Notes
- The relay spool is durable file-backed state, not an in-memory queue.
- The delivery lifecycle and retry semantics are described in `docs/architecture/service-api-delivery-flow.md`.
- This slice is intentionally narrower than the broader working vertical slice. It exists to prove durable cross-node relay and restart behavior specifically.
