# Live Chain-Backed Bridge Finality Slice

This runbook records one bounded service-api bridge finality proof backed by a
real Solana devnet transaction. The service persists a canonical receipt before
returning terminal state, and a separate RPC query verifies the exact
transaction signature and finalized slot.

## What This Slice Proves

- the transaction is bound to the bridge ID, source message ID, target network,
  and SHA-256 payload digest
- the prepared signed transaction is persisted before submission
- a real Solana devnet transaction reaches `finalized` commitment
- independent RPC verification matches the persisted signature and slot
- retry and restart reconcile before resubmit and return the same receipt
- the observed recipient balance delta equals exactly one configured transfer

## Evidence

The secret-safe artifact is
`docs/validation/evidence/7160-live-bridge-finality.json`. It records public
receipt fields, the finalized slot, independent verification result, restart
receipt match, and recipient balance delta. It contains no signer path, private
key, seed phrase, or credential.

The artifact was generated on 2026-07-24 with:

```bash
KAMN_LIVE_BRIDGE_PROOF_OUTPUT="$PWD/docs/validation/evidence/7160-live-bridge-finality.json" \
  cargo test -p kamn-node --bin kamn-node \
  integration_live_bridge_receipt_matches_independent_finalized_rpc_evidence \
  -- --ignored --nocapture
```

Expected result: one test passes, the artifact reports
`independent_rpc_verified: true`, `restart_receipt_matched: true`, and
`exactly_one_transfer_observed: true`.

## Failure Semantics

- pending or absent finality never writes terminal state
- an ambiguous submission returns `BRIDGE_RECONCILIATION_REQUIRED`
- malformed or mismatched evidence returns `BRIDGE_FINALITY_EVIDENCE_INVALID`
- receipt or prepared-signature reuse returns `BRIDGE_RECEIPT_REPLAY`

## Bounded Interpretation

This is not generalized cross-chain finality and not production readiness. It
does not prove mainnet custody, arbitrary bridge protocols, consensus, economic
security, or multi-driver settlement parity.
