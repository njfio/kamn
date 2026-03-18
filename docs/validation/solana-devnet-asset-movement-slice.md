# Solana Devnet Asset-Movement Slice

This runbook documents one bounded live Solana devnet asset-movement lane on current `main`. It moves beyond slot-derived evidence by requiring real Solana devnet transaction submission from the public service-api escrow release path and by persisting the resulting `settlement_tx_signature` plus bounded reconciliation metadata.

## What This Proves
- the service-api escrow release path can submit one real Solana devnet transaction when the live settlement lane is explicitly configured
- the release response persists `settlement_tx_signature`, `settlement_network=solana:devnet`, and `settlement_commitment=finalized`
- repeated release for the same escrow reuses the same persisted transaction linkage instead of submitting a second transfer
- after restart, repeated release for the same escrow returns the same persisted Solana signature instead of creating a new transfer
- the bounded live proof can use a real devnet-funded signer and observe recipient balance movement after release

## What This Does Not Prove
- not broad production readiness
- not generalized multi-chain settlement
- not bridge settlement
- not Byzantine-safe dispute resolution
- not a full custody architecture

## Required Configuration
- `KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL=https://api.devnet.solana.com`
- `KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE=/path/to/devnet-sender.json`
- `KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY=<solana-recipient-pubkey>`
- `KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS=<positive-lamport-amount>`
- optional: `KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT=finalized`

## Stable Regression Commands
```bash
cargo test -p kamn-node --test solana_devnet_asset_movement_slice_contract -- --nocapture

cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::task_escrow_persistence_contract_tests::task_escrow_solana_asset_movement_contract_tests::integration_service_api_endpoint_live_solana_asset_movement_lane_fails_at_startup_when_recipient_env_missing' \
  -- --exact --nocapture

cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::task_escrow_persistence_contract_tests::task_escrow_solana_asset_movement_contract_tests::integration_service_api_endpoint_live_solana_asset_movement_release_persists_transaction_signature_metadata' \
  -- --exact --nocapture

cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::task_escrow_persistence_contract_tests::task_escrow_solana_asset_movement_contract_tests::integration_service_api_endpoint_live_solana_asset_movement_release_is_idempotent_for_repeated_submit' \
  -- --exact --nocapture

cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::task_escrow_persistence_contract_tests::task_escrow_solana_asset_movement_contract_tests::integration_service_api_endpoint_live_solana_asset_movement_release_reuses_signature_after_restart' \
  -- --exact --nocapture
```

## Live Devnet Proof Command
This command performs real Solana devnet transaction submission and requires live RPC plus airdrop availability:

```bash
cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::task_escrow_persistence_contract_tests::task_escrow_solana_asset_movement_live_contract_tests::integration_service_api_endpoint_live_solana_asset_movement_release_submits_real_devnet_transfer' \
  -- --ignored --exact --nocapture
```

## Expected Evidence
- `state=released`
- non-empty base58 `settlement_tx_signature`
- `settlement_network=solana:devnet`
- `settlement_commitment=finalized`
- repeated release returns the same `settlement_tx_signature`
- release after restart returns the same persisted `settlement_tx_signature`
- the live ignored proof observes recipient balance growth by the configured lamports amount

## Bounded Interpretation
This slice proves one bounded live Solana devnet asset-movement lane on current `main`. It is intentionally scoped to one public KAMN release path and one explicit proof configuration. It does not claim broad production settlement semantics or full operational hardening.
