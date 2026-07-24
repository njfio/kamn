# Pi/MCP Service-Receipt Authority Slice

This runbook documents one bounded canonical three-role transaction on merged
`main`. Its authority comes from service-persisted receipts collected through
three independent Pi-driven MCP actors, not from actor-authored local evidence.

## What This Proves

- `make demo-agent-transaction` runs the canonical transaction path.
- Each mutation response uses `kamn.mcp.authority-receipt.v1`.
- The durable transcript uses `kamn.service.receipt-chain.v1`.
- The receipt order is:
  `task:create -> task:accept -> escrow:fund -> task:complete` followed by
  `escrow:release-authorize -> settlement:confirmed`.
- All three actors bind the same task, transaction, escrow, finalized
  settlement signature, receipt-chain commitment, and public commitment.
- Agent A and Agent B retain participant-private views while Agent C receives a
  restricted-public view with zero participant-private fields.
- Settlement evidence uses `live-service-persisted-receipt` and is reconciled
  against independent finalized Solana devnet RPC evidence.
- A standalone verifier runs after the Pi, MCP, node, and supervisor children
  exit and recomputes the durable authority and settlement bindings.

## What This Does Not Prove

- not production readiness
- not mainnet or custody safety
- not real economic value beyond Solana devnet test tokens
- not broad bridge finality
- not generalized external settlement
- not multi-authority release or Byzantine consensus

## Stable Regression Commands

```bash
cargo test -p kamn-e2e-harness \
  --test mvp_demo_runtime_receipt_chain_contract -- --nocapture
cargo test -p kamn-e2e-harness \
  --test agent_transaction_demo_success_contract -- --nocapture
cargo test -p kamn-e2e-harness \
  --test independent_agent_transaction_verifier_contract -- --nocapture
cargo test -p kamn-node \
  --test pi_mcp_service_receipt_authority_slice_contract -- --nocapture
```

## Live Command

From a fresh checkout with the documented required Solana devnet
configuration:

```bash
make demo-agent-transaction
cargo run -p kamn-e2e-harness -- \
  verify-mvp-demo --report .kamn/demo/latest/proof/report.json
```

The command must report `GO`, and the post-child standalone verifier must report
`PASS`. A required devnet run hard-fails when persisted service authority,
receipt-chain membership, transaction binding, or independent RPC evidence is
missing or inconsistent.

## Published Evidence

The secret-safe fresh-checkout proof is recorded at
`docs/validation/evidence/7126-fresh-checkout-pi-mcp-service-authority.md`.
That note identifies the exact merged source commit, run ID, commitments, and
finalized transaction facts without retaining keys, environment values,
transient paths, or participant-private payloads.

## Interpretation

This is one executable bounded proof of service-backed Pi/MCP transaction
authority. It is stronger than an ambient actor-evidence claim because the
verifier recomputes membership in the durable ordered service receipt chain and
binds the finalized settlement to the same transaction. It remains a devnet
evaluation slice, not a production or generalized settlement guarantee.
