# 7126 Fresh-Checkout Pi/MCP Service Authority Evidence

## Provenance

- source_commit: 732b1d59027724632af16c4073ca4f1cbe27db31
- clone_source: origin/main
- clone_state: fresh
- inherited_target: false
- inherited_kamn_state: false
- run_id: run-63361-1784859146830
- make demo-agent-transaction: GO
- standalone_verifier: PASS
- children_exited_before_verifier: true

## Service Authority

- authority_schema: kamn.mcp.authority-receipt.v1
- receipt_chain_schema: kamn.service.receipt-chain.v1
- execution_surface: live-service-persisted-receipt
- receipt_chain_commitment: sha256:eff3a939e3e3ba09d7ac33038d83a4a8c4ad190737f81a3d13ac8e7e8a67cf4e
- service_receipt_commitment: sha256:618ebbe47a8520daba9399034d082324c82e73f30fe1360c5709617a4ee76e90
- ordered_actions: task:create, task:accept, escrow:fund, task:complete,
  escrow:release-authorize, settlement:confirmed
- agent_a_view: participant-private
- agent_b_view: participant-private
- agent_c_view: restricted-public
- agent_c_participant_private_field_count: 0

All three actor observations bind the same task, transaction, escrow, finalized
signature, receipt-chain commitment, and public commitment. Their report-bound
observation receipt digests remain distinct.

## Independent Solana Evidence

- network: solana:devnet
- settlement_commitment: finalized
- settlement_tx_signature:
  2VkJPvLPjK34KJd2LK294reNc1xFZdNif4P7wsB7VBA8vC1hnm5QrrSuxMJUJDsdGHBDiucvAUyRoanmveBiTBuq
- finalized_slot: 478456906
- transaction_error: null
- fee_lamports: 5000
- amount_lamports: 1000000
- recipient_balance_before_lamports: 9000000
- recipient_balance_after_lamports: 10000000
- recipient_balance_delta_lamports: 1000000
- transfer_count: 1
- retry_duplicate_count: 0
- rpc_verification: GO
- explorer:
  https://explorer.solana.com/tx/2VkJPvLPjK34KJd2LK294reNc1xFZdNif4P7wsB7VBA8vC1hnm5QrrSuxMJUJDsdGHBDiucvAUyRoanmveBiTBuq?cluster=devnet

## Hygiene

- transcript_sha256:
  3d58a6768e24bdb0d326861f8a41aded6e49c323251181b05701b03ef050cd0d
- transcript_and_proof_hygiene_scan: PASS
- generated_agent_secret_scan: PASS

This note contains no key material, environment values, transient checkout
paths, or participant-private payloads.

## Claim Boundary

This proves one local KAMN three-agent transaction backed by durable service
authority and one finalized Solana devnet test-token transfer.

- not production readiness
- not mainnet
- not custody
- not broad bridge finality
- not generalized external settlement
- devnet test tokens have no real economic value
