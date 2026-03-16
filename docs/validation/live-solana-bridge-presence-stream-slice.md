# Live Solana Bridge Presence-Stream Slice

This runbook documents one bounded live Solana-backed bridge presence-stream slice on current `main`. It proves that the existing live bridge forward lane reaches the presence-mode websocket surface after the initial presence snapshot and emits a `service-api.bridge.forwarded` event with non-placeholder bridge evidence. It does not prove live on-chain settlement, bridge finality, or external economic settlement.

## Scope
- presence-mode websocket on `/v1/events/ws`
- initial `m9.presence.snapshot` projection plus later `service-api.bridge.forwarded` event
- service-api bridge submit and forward on the current node path
- live Solana-backed bridge evidence enabled through `KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL`

## Proof Anchors
This slice is grounded in current-main tests:
- `integration_service_api_endpoint_websocket_presence_mode_streams_live_bridge_projection_event`
- `cargo test -p kamn-node --test live_solana_bridge_presence_stream_slice_contract -- --nocapture`

## What This Proves
- the presence-mode websocket surface returns the expected presence snapshot first
- the same presence-mode websocket stream remains open long enough to observe a later live bridge forward
- the forwarded presence-stream event is `service-api.bridge.forwarded`
- the forwarded event carries non-placeholder `target_message_id` and `forward_tx_hash`
- the proof uses the real websocket presence consumer plus the real bridge submit and forward routes on current `main`

## What This Does Not Prove
- not live on-chain settlement
- not bridge finality
- not external economic settlement
- not multi-node consensus or arbitrary partition tolerance

## Operator Commands
Run the exact proof targets from a clean checkout:

```bash
cargo test -p kamn-node --test live_solana_bridge_presence_stream_slice_contract -- --nocapture
cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::websocket_contract_tests::presence_projection_contract_tests::live_bridge_presence_stream_contract_tests::integration_service_api_endpoint_websocket_presence_mode_streams_live_bridge_projection_event' \
  -- --exact --nocapture
```

## Expected Evidence
- websocket frames begin with `m9.presence.snapshot`
- later websocket frames include `service-api.bridge.forwarded`
- `target_message_id` is present and non-placeholder
- `forward_tx_hash` is present and non-placeholder
- live Solana bridge configuration remains anchored to `KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL`

## Notes
- This slice deepens the existing live bridge websocket proof by proving the live Solana-backed bridge evidence lane reaches the presence-mode websocket surface.
- It does not extend the claim to settlement, finality, or economic movement.
