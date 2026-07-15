# Fresh-Checkout MVP Evaluator Evidence

decision: GO
issue: 7123
clone_source: origin/main
clone_head: 6428f9f88a9aab4af248753c994070042416cebe
clone_state: fresh
inherited_target: false
inherited_kamn_state: false
run_id: run-32638-1784027703311
actor_driver: pi
pi_version: 0.80.3
pi_model: openai-codex/gpt-5.5
devnet_mode: required

## Commands And Shutdown

- `make demo-agent-transaction`: GO
- `cargo run -p kamn-e2e-harness -- verify-mvp-demo --report
  .kamn/demo/latest/proof/report.json`: PASS
- children_exited_before_verifier: true
- transcript_sha256:
  `79a5ae6ce1b36056e73f5e5ba161cb8323b5d46ecc401aab5ab0f7e59621ccd9`
- Transcript hygiene: no user path, key-file value, private-key marker, or
  sensitive environment assignment was present. The transcript remains outside
  git; only its digest is published.

## Independent Devnet Verification

- rpc_verification: GO
- network: solana:devnet
- commitment: finalized
- signature:
  `29ct6LCWQx5L9sEQ3WSWoBVbQReRP1gpVcbcPXm7D1UjYYMjannkr8s58AaMhYbhRdFv2yYRWupdYynd1TbjLdRh`
- slot: 476189316
- transaction_error: none
- payer: `2FjUiacAXtokhA8YzGiyfVEdu5D9LxKFhjptJLrz4V9T`
- recipient: `FV5LvudLjZQGCrPwXUY2JaVr26sQE15K25BGvsKWvyFe`
- amount_lamports: 1000000
- fee_lamports: 5000
- recipient_balance_before_lamports: 2546000000
- recipient_balance_after_lamports: 2547000000
- recipient_balance_delta_lamports: 1000000
- transfer_count: 1
- retry_duplicate_count: 0
- Explorer:
  https://explorer.solana.com/tx/29ct6LCWQx5L9sEQ3WSWoBVbQReRP1gpVcbcPXm7D1UjYYMjannkr8s58AaMhYbhRdFv2yYRWupdYynd1TbjLdRh?cluster=devnet

The transaction was fetched independently from Solana devnet after the demo
and standalone verifier exited. The proof-retry test reused persisted evidence
without submission, and the restart-monotonicity test returned the confirmed
settlement on retry without submitting a second transfer.

## Actor And Product Story

- distinct_authenticated_actor_count: 3
- Agent A DID suffix: `keyh-2d6f4037ad707813f4b82f0fae4401e1`
- Agent B DID suffix: `keyh-74de722999161b3b42fdd016decb7ae1`
- Agent C DID suffix: `keyh-823ad243e71d3bc1aea4032440eef59d`
- task_lifecycle: GO
- escrow_lifecycle: GO
- durable_receipts: GO
- relay_projection: GO
- websocket_visibility: GO
- audit_export: GO
- Transcript actions: register A/B/C, create task, accept task, fund escrow,
  complete task, release escrow, query A/B participant projections, and query
  C verifier projection.

## Disclosure Comparison

- agent_a_view: participant-private
- agent_a_private_field_count: 3
- agent_b_view: participant-private
- agent_b_private_field_count: 3
- agent_c_view: restricted-public
- agent_c_participant_private_field_count: 0
- All three views bind the same task, escrow, terms, amount, finalized
  signature, and public-view digest. A and B have distinct redacted private-view
  digests; C receives neither participant-private digest nor raw private data.

## Claim Boundary

This is a real local KAMN runtime and a Solana-devnet-backed transfer using
devnet test tokens. It is not production, not mainnet, not custody, not real
economic value, and not a generalized exchange.

The current report records `execution_surface: command-override` for both
settlement claims. The transfer itself is finalized and independently verified,
but direct persisted service-receipt collection is not claimed here and remains
the required follow-up in the dependent settlement-evidence issue.
