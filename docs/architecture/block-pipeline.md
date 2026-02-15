# Block Pipeline Architecture

This document captures Phase 3.2 initial core delivery for mempool block
production and consensus validation (Task #2926, Subtask #2927).

## Scope

- Add explicit mempool -> listener quorum -> approver quorum -> block commit
  pipeline contracts in `kamn-core`.
- Keep deterministic fail-closed outcomes for consensus mismatch and empty
  mempool conditions.
- Wire processor runtime planning so consensus validation is explicitly visible
  in runtime component topology.

## Core Components

- `MempoolBlockPipeline`
  - orchestrates one consensus round over pending mempool transactions.
- `GossipIngressAdapter`
  - decodes deterministic gossip frame payloads into typed
    `BaselineTransaction` and `CanonicalCommitRecord` structures.
- `GossipFrameTransportMempoolFeed`
  - bridges `PeerGossipFrame` ingress payloads into transport-fed mempool
    candidates while retaining decoded canonical block candidates.
- `BlockConsensusRoundInput`
  - listener and approver attestation input envelope for a round.
- `BlockPipelineCommitReport`
  - committed block + listener/approver decisions + payload digest.
- `BlockPipelineError`
  - typed fail-closed errors for listener/approver/smoke failures and
    deterministic digest mismatches.

## Consensus Round Flow

1. Snapshot processor mempool transactions.
2. Compute deterministic payload digest from ordered transaction identity.
3. Validate listener quorum using `ListenerQuorumEvaluator`.
4. Validate approver quorum using `ApproverQuorumEvaluator`.
5. Commit produced block through `RoleSmokeNetwork::produce_block`.

Commit is blocked if either quorum validation fails.

## Runtime Wiring Integration

Processor role runtime wiring now includes:

- `mempool`
- `executor`
- `block-producer`
- `consensus-validator`

## Deterministic Guardrails

- Empty mempool is rejected before consensus evaluation:
  `BlockPipelineError::EmptyMempool`.
- Approver payload-digest overrides must match deterministic digest:
  `BlockPipelineError::ConsensusPayloadDigestMismatch`.
- Listener and approver quorum errors are surfaced as typed failures.
- Unsupported ingress topics fail closed with:
  `p2p_ingress_topic_unsupported`.
- Malformed ingress payload key/value lines fail closed with:
  `p2p_ingress_payload_line_malformed`.
- Invalid transaction signatures fail closed with:
  `p2p_ingress_tx_signature_invalid`.

Regression marker:
- `Regression: #2927` keeps digest mismatch fail-closed before commit.
- `Regression: #3415` keeps gossip ingress decode/reason-code taxonomy stable.

## Validation Commands

```bash
cargo test -p kamn-core --test block_pipeline
cargo test -p kamn-core --test block_pipeline_gossip_ingest
cargo test -p kamn-core block_pipeline
cargo clippy -p kamn-core -- -D warnings
cargo fmt --check
```

## Live Validation

Use these deterministic lane commands to validate integration evidence:

```bash
scripts/runtime/validate_block_pipeline_live.sh
scripts/runtime/test_validate_block_pipeline_live.sh
```

Expected live markers:
- `status=pass`
- `final_decision=GO`
- `block_pipeline_contract_status=verified`
- `docs_contract_status=verified`
- `fail_closed_status=verified`
- `fail_closed_reason_code=block_pipeline_payload_digest_mismatch`
