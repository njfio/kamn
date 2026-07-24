# 7159 Pi/MCP Service-Receipt Authority Proof Index

## Objective

Publish one bounded, operator-readable proof slice for the canonical
`make demo-agent-transaction` path and index it from the current runtime proof
catalog without expanding the claims already proven by issue #7126.

## Inputs / Outputs

Inputs:
- the merged canonical demo behavior on commit
  `732b1d59027724632af16c4073ca4f1cbe27db31`
- the fresh-checkout run `run-63361-1784859146830`
- existing executable Pi/MCP authority and receipt-chain contracts

Outputs:
- a dedicated validation runbook
- a redacted fresh-checkout evidence note
- one proof-index entry
- hard-fail Rust documentation contracts for the slice and evidence note

## Boundaries / Non-Goals

- Do not change runtime behavior.
- Do not reuse the #7123 command-override rehearsal as service-authority proof.
- Do not claim production readiness, mainnet custody, broad bridge finality,
  generalized external settlement, or multi-authority release.
- Do not publish secrets, environment values, key material, transient absolute
  paths, or private actor receipt payloads.
- Do not represent a bounded Solana devnet transfer as real economic value.

## Failure Modes

- The runtime proof index omits the canonical demo command or dedicated runbook.
- The runbook omits service authority, receipt-chain membership, settlement
  binding, or independent post-child verification.
- The evidence note does not identify the exact source commit and fresh run.
- The evidence note leaks secret-bearing material or transient paths.
- The published proof overstates its runtime, custody, bridge, or settlement
  boundaries.
- A stale command-override artifact is presented as persisted service authority.

## Acceptance Criteria

- [ ] The runtime proof index links the dedicated Pi/MCP authority slice and
      names `make demo-agent-transaction`.
- [ ] The indexed claim is limited to one three-role Pi/MCP transaction whose
      authority comes from durable service receipts.
- [ ] The runbook identifies `kamn.mcp.authority-receipt.v1`,
      `kamn.service.receipt-chain.v1`, the ordered service actions, shared public
      commitments, actor-view boundaries, finalized settlement reconciliation,
      and a standalone verifier after child exit.
- [ ] The evidence note binds the exact merged commit, fresh run ID, canonical
      command result, verifier result, receipt commitments, finalized devnet
      transaction facts, one transfer, and zero retry duplicates.
- [ ] The evidence note contains no secrets, key material, environment values,
      or transient absolute paths.
- [ ] Focused Rust contracts fail if required proof anchors or non-claims drift.
- [ ] Existing canonical demo and runtime proof-index contracts remain green.

## Files To Touch

- `specs/7159-pi-mcp-service-receipt-authority-proof-index.md`
- `docs/validation/pi-mcp-service-receipt-authority-slice.md`
- `docs/validation/evidence/7126-fresh-checkout-pi-mcp-service-authority.md`
- `docs/validation/current-proven-runtime-slices.md`
- `crates/kamn-node/tests/pi_mcp_service_receipt_authority_slice_contract.rs`
- `crates/kamn-e2e-harness/tests/pi_mcp_service_authority_evidence_contract.rs`

## Error Semantics

The proof contracts use hard assertions with a marker-specific failure message.
Missing files, missing anchors, stale execution-surface language, leaked path
markers, or missing non-claims fail the test. There is no fallback proof source.

## Test Plan

Red:
- Add the slice contract before the runbook and index entry exist.
- Add the evidence contract before the evidence note exists.
- Run each focused target and capture the expected failures.

Green:
- Add the dedicated runbook, redacted evidence note, and proof-index entry.
- Re-run both focused contracts.

Refactor:
- Remove duplicated prose where a stable link or evidence table is clearer.
- Verify file and function size, formatting, names, and boundary language.

Integration:
- Run `runtime_proof_index_contract`.
- Run the existing canonical demo runbook and authority contracts.
- Run the touched crate test targets and formatting checks.
