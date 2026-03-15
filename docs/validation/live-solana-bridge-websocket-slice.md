# Live Solana Bridge Websocket Slice

This runbook documents one bounded live Solana-backed bridge websocket slice on current `main`. It proves that the existing live bridge forward lane reaches the real websocket event stream and emits a `service-api.bridge.forwarded` event with non-placeholder bridge evidence. It does not claim live on-chain settlement, bridge finality, or external economic settlement.

## Scope
- websocket upgrade on `/v1/events/ws`
- service-api bridge submit and forward on the current node path
- live Solana-backed bridge evidence enabled through `KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL`
- forwarded websocket event projection carrying non-placeholder `target_message_id` and `forward_tx_hash`

## Proof Anchors
This slice is grounded in current-main tests:
- `integration_service_api_endpoint_websocket_streams_live_bridge_forwarded_event_after_upgrade`
- `cargo test -p kamn-node --test live_solana_bridge_websocket_slice_contract -- --nocapture`

## What This Proves
- the websocket event stream remains open long enough to observe the live bridge forward on the same server instance
- a live Solana-backed bridge forward emits `service-api.bridge.forwarded` on the websocket event stream
- the forwarded websocket event carries non-placeholder `target_message_id` and `forward_tx_hash` values
- the proof uses the real websocket upgrade plus the real bridge submit/forward routes on current `main`

## What This Does Not Prove
- not live on-chain settlement
- not bridge finality beyond the existing bounded proofs
- not external economic settlement
- not multi-node consensus or arbitrary partition tolerance

## Operator Commands
Run the exact proof targets from a clean checkout:

```bash
cargo test -p kamn-node --test live_solana_bridge_websocket_slice_contract -- --nocapture
cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::websocket_contract_tests::upgrade_delivery_contract_tests::live_bridge_delivery_contract_tests::integration_service_api_endpoint_websocket_streams_live_bridge_forwarded_event_after_upgrade' \
  -- --exact --nocapture
```

## Expected Evidence
- websocket frames include `service-api.bridge.forwarded`
- `target_message_id` does not match `msg-bridge-target-{bridge_id}`
- `forward_tx_hash` does not match `sha256:bridge-forwarded-{bridge_id}`
- live Solana bridge configuration is still anchored to `KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL`

## Notes
- This slice deepens the existing live bridge dispatch proof by showing the live-backed evidence reaches a downstream consumer surface.
- It does not extend the claim to settlement or finality.
