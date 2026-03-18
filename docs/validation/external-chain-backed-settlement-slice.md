# External-Chain-Backed Settlement Slice

This runbook documents one bounded external-chain-backed settlement slice on current `main`. It proves one service-api escrow release lane where the existing live Solana devnet proof path is consumed before the terminal escrow state is emitted and persisted. It does not prove external economic settlement, bridge finality, or on-chain asset movement.

## Scope
- one real service-api escrow release path on current `main`
- one startup-selected live Solana RPC evidence source through `KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL`
- one persisted `settlement_receipt_hash` linkage that survives restart

## Proof Anchors
- `crates/kamn-node/src/service_api_endpoint/live_settlement_dispatch.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/task_escrow_ops.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests/task_escrow_live_settlement_contract_tests.rs`
- `crates/kamn-node/tests/external_chain_backed_settlement_slice_contract.rs`

## What This Proves
- `integration_service_api_endpoint_live_settlement_release_persists_external_receipt_linkage` exercises a real release path with live Solana RPC configured
- the release response carries a non-placeholder `settlement_receipt_hash`
- the same external-chain-backed escrow settlement lane persists that receipt linkage across restart
- current `main` now has one bounded external-chain-backed escrow settlement lane on the public service-api surface

## What This Does Not Prove
- not external economic settlement
- does not prove bridge finality
- does not prove live bridge settlement
- does not prove Solana asset transfer or custody movement
- does not prove Byzantine-safe dispute resolution

## Operator Commands
Run the bounded docs contract:

```bash
cargo test -p kamn-node --test external_chain_backed_settlement_slice_contract -- --nocapture
```

Run the real settlement-lane integration proof:

```bash
KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL=https://api.devnet.solana.com \
cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::task_escrow_persistence_contract_tests::task_escrow_live_settlement_contract_tests::integration_service_api_endpoint_live_settlement_release_persists_external_receipt_linkage' \
  -- --exact --nocapture
```

## Expected Evidence
- the release response includes `state=released`
- the release response includes a non-empty `settlement_receipt_hash`
- the persisted escrow record keeps the same `settlement_receipt_hash` after restart
- placeholder receipt material is rejected by the proof contract because the hash must be non-empty and non-placeholder

## Notes
- external-chain-backed settlement slice: `docs/validation/external-chain-backed-settlement-slice.md`
- This slice is intentionally bounded to evidence-coupled settlement semantics on the service-api surface.
- It uses the existing live Solana proof lane as the external evidence source.
