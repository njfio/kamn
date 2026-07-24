# Bridge-Authorized Escrow Settlement Slice

This runbook documents one bounded release path where escrow settlement is
authorized by a finalized bridge receipt that is already bound to the same
escrow, task, actor, recipient, amount, asset, network, and terms digest.

## What This Slice Proves

- exactly one economic transfer is observed and escrow release does not submit a
  second settlement transaction
- a finalized bridge receipt can authorize release only when its bound economic
  terms match the escrow and task records
- cross-resource receipt, cross-actor receipt, tampered digest, and replayed
  receipt attempts fail closed
- authority validation precedes settlement evidence validation and terminal
  persistence
- participant and verifier projections expose the same bridge-to-settlement
  receipt chain

## Deterministic Verification

Run the focused deterministic contracts:

```bash
cargo test -p kamn-node --test bridge_authorized_escrow_settlement_contract --no-run
cargo check -p kamn-node --tests
```

Focused runtime coverage lives in:

- `task_escrow_bridge_authority_contract_tests.rs`
- `task_projection_settlement_contract_tests.rs`
- `task_projection_receipt_chain_contract_tests.rs`

## Live Proof

The bounded live proof depends on the finalized bridge lane from issue `#7160`
and reuses that bridge receipt during escrow release. The intended proof command
is:

```bash
KAMN_LIVE_BRIDGE_PROOF_OUTPUT="$PWD/docs/validation/evidence/7160-live-bridge-finality.json" \
  cargo test -p kamn-node --bin kamn-node \
  integration_live_bridge_receipt_matches_independent_finalized_rpc_evidence \
  -- --ignored --nocapture
```

Then run the bridge-authorized release proof on the same configured Solana
devnet recipient and signer to confirm the persisted bridge receipt digest and
bridge transaction signature survive release, retry, and restart.

## Bounded Interpretation

This is not generalized settlement and not production readiness. It does not
claim mainnet custody, generalized bridge consensus, adapter parity, or any
economic guarantee beyond one bounded finalized Solana devnet transfer.
