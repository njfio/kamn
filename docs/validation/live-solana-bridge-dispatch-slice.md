# Live Solana Bridge Dispatch Slice

This runbook documents one bounded live Solana-backed bridge dispatch slice on current `main`. It proves that the service-api bridge forward path can derive non-placeholder bridge evidence from a real Solana devnet JSON-RPC interaction and persist that evidence across restart. It does not claim on-chain transaction submission, bridge finality, or external settlement.

## What This Slice Proves
- `KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL` enables one bounded live bridge lane and fails at startup when configured with an empty value
- the service-api bridge forward path invokes the checked-in live Solana devnet proof runner instead of persisting placeholder bridge values
- the forward response persists non-synthetic `target_message_id` and `forward_tx_hash` values
- the persisted live-backed evidence remains queryable after restart

## What This Slice Does Not Prove
- not on-chain Solana transaction submission
- not live bridge settlement
- not bridge finality
- not external economic settlement
- not broad production readiness

## Commands
```bash
cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::bridge_persistence_restart_contract_tests::bridge_persistence_restart_contract_tests::integration_service_api_endpoint_live_bridge_lane_fails_at_startup_for_empty_rpc_url' \
  -- --exact --nocapture

cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::bridge_persistence_restart_contract_tests::bridge_persistence_restart_contract_tests::integration_service_api_endpoint_live_bridge_forward_path_rejects_placeholder_evidence' \
  -- --exact --nocapture

cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::bridge_persistence_restart_contract_tests::bridge_persistence_restart_contract_tests::integration_service_api_endpoint_live_bridge_forward_evidence_survives_restart' \
  -- --exact --nocapture
```

## Expected Evidence
- startup fails loudly when `KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL` is empty
- forward responses stay `HTTP/1.1 200 OK`
- `target_message_id` no longer matches `msg-bridge-target-{bridge_id}`
- `forward_tx_hash` no longer matches `sha256:bridge-forwarded-{bridge_id}`
- restart-visible bridge state preserves the same non-placeholder evidence

## Bounded Interpretation
This slice proves a bounded live Solana-backed bridge evidence lane on the service-api path. The live Solana interaction is used to derive forward evidence, not to claim live on-chain submission or settlement.
