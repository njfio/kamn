# Solana Receipt Finality Slice

This runbook documents one current KAMN bounded Solana receipt-finality slice on `main`. It proves deterministic Solana receipt-finality normalization on the public core surface. It does not claim live Solana devnet proof or live chain-backed settlement.

## Scope
- deterministic Solana receipt-finality normalization on the public core surface
- typed rejection for unsupported Solana finality labels

## Proof Anchors
This slice is grounded in current-main tests:
- `solana_finalized_receipt_normalizes_to_final`
- `solana_processed_receipt_normalizes_to_pending`
- `solana_failed_receipt_normalizes_to_failed`
- `regression_rejects_unknown_solana_finality_label`

## What This Proves
- Solana `finalized` receipts normalize to `Final`
- Solana `processed` receipts normalize to `Pending`
- Solana failed receipts remain fail-closed regardless of label
- unsupported Solana finality labels return typed `UnsupportedFinalityLabel` errors

## What This Does Not Prove
- not live Solana devnet proof
- not live chain-backed settlement
- not bridge quorum behavior on a real network
- not external economic settlement

## Operator Commands
Run the exact bounded Solana receipt-finality proof target from a clean checkout:

```bash
cargo test -p kamn-core --test cross_chain_receipt_finality -- --nocapture
```

## Expected Evidence
- Solana `finalized`, `processed`, failed, and invalid-label cases execute on the public core normalization surface
- invalid Solana labels fail with typed normalization errors instead of permissive fallback behavior

## Notes
- This slice proves bounded Solana receipt-finality normalization on the public core surface.
- A future devnet issue should prove live Solana evidence separately.
