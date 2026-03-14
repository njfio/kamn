# Bridge Finality Slice

This runbook documents one current KAMN bridge-finality-related slice on `main` that proves deterministic receipt-finality normalization and persisted forwarded bridge state. It is deliberately narrower than live chain-backed bridge finality or external settlement claims.

## Scope
- deterministic Ethereum and Near receipt-finality normalization on the public core surface
- persisted `forwarded` bridge state across restart on the current service-api path

## Proof Anchors
This slice is grounded in current-main tests:
- `ethereum_finalized_receipt_normalizes_to_final`
- `near_final_receipt_normalizes_to_final`
- `direct_pending_status_normalizes_to_pending_through_core_surface`
- `regression_rejects_unknown_near_finality_label`
- `integration_service_api_endpoint_persists_bridge_state_across_restart`

## What This Proves
- deterministic receipt-finality normalization maps Ethereum `finalized` and Near `final` receipts to `Final`
- pending and failed receipt states remain fail-closed on the public core surface
- unknown Near finality labels are rejected rather than normalized loosely
- persisted forwarded bridge state remains queryable after restart on the current service-api storage path
- the current node path records `bridge_status=forwarded` and a stable `forward_tx_hash` across restart

## What This Does Not Prove
- not live chain-backed bridge finality
- not external settlement
- not bridge quorum enforcement on a real network
- not cross-chain economic settlement
- not production deployment readiness

## Operator Commands
Run the exact bounded bridge-finality proof targets from a clean checkout:

```bash
cargo test -p kamn-core --test cross_chain_receipt_finality -- --nocapture
cargo test -p kamn-node integration_service_api_endpoint_persists_bridge_state_across_restart -- --exact --nocapture
```

## Expected Evidence
- core receipt-finality evidence shows final, pending, failed, and invalid-label handling on the public normalization surface
- node bridge evidence shows a persisted bridge record reaches and retains `forwarded` state with stable target message and forward hash across restart

## Notes
- This slice proves deterministic receipt-finality normalization and persisted forwarded bridge state.
- It does not prove live chain-backed bridge finality or external settlement semantics.
