# Live Solana Devnet Proof Slice

This runbook documents one bounded live Solana devnet proof slice on current `main`. It proves live Solana devnet JSON-RPC commitment evidence and drives that evidence through KAMN's public `normalize_cross_chain_receipt(...)` surface. It does not claim live bridge settlement, live message relay, or external economic finality.

## What This Slice Proves
- `scripts/runtime/run_live_solana_devnet_proof.py` performs real Solana devnet JSON-RPC requests against `https://api.devnet.solana.com`
- the lane captures `getHealth`, `getVersion`, and `getSlot` evidence for `processed`, `confirmed`, and `finalized`
- the validator fails closed if the live report is malformed or if `processed >= confirmed >= finalized` does not hold
- `cargo test -p kamn-core --test live_solana_devnet_receipt_normalization -- --nocapture` normalizes the observed live Solana labels through `normalize_cross_chain_receipt`
- the policy checker returns GO only when both the live Solana report and the normalization report are valid

## What This Slice Does Not Prove
- not live bridge settlement
- not live message relay
- not external economic settlement
- not a live KAMN Solana bridge dispatch
- not broad production readiness

## Commands
```bash
python3 scripts/runtime/run_live_solana_devnet_proof.py \
  --rpc-url https://api.devnet.solana.com \
  --output-json /tmp/live-solana-devnet-report.json

python3 scripts/runtime/validate_live_solana_devnet_proof.py \
  --report-file /tmp/live-solana-devnet-report.json \
  --output-json /tmp/live-solana-devnet-validation.json

KAMN_SOLANA_DEVNET_REPORT_FILE=/tmp/live-solana-devnet-report.json \
KAMN_SOLANA_DEVNET_NORMALIZATION_REPORT=/tmp/live-solana-devnet-normalization.json \
  cargo test -p kamn-core --test live_solana_devnet_receipt_normalization -- --nocapture

python3 scripts/runtime/check_live_solana_devnet_proof_policy.py \
  --validation-report-file /tmp/live-solana-devnet-validation.json \
  --normalization-report-file /tmp/live-solana-devnet-normalization.json \
  --expected-final-decision GO \
  --output-json /tmp/live-solana-devnet-policy.json
```

## Expected Evidence
- `status=ok` from the live runner
- `health_status=ok` from Solana devnet JSON-RPC
- integer slots for `processed`, `confirmed`, and `finalized`
- `slot_order_valid=true`
- a normalization artifact showing:
  - `processed -> Pending`
  - `confirmed -> Pending`
  - `finalized -> Final`
- `live_solana_devnet_proof_policy_passed`

## Bounded Interpretation
This slice proves bounded live Solana devnet JSON-RPC evidence normalized through the public receipt surface. It is a live finality-observation proof lane, not a live settlement or live relay proof lane.
