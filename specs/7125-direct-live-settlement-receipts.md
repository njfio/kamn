# Issue 7125: Direct Live Settlement Receipts

## Objective

Make canonical required-devnet MVP success consume the durable settlement
record written by the live KAMN service, independently reconcile it with Solana
RPC and the three runtime actor projections, and prevent `command-override`
evidence from satisfying canonical `GO`.

## Inputs And Outputs

Inputs:

- The canonical supervisor's persisted `service-api-state.json` after runtime
  shutdown.
- The three validated Pi actor artifacts and live task evidence paths.
- The configured payer public key, recipient, amount, RPC URL, and finalized
  Solana confirmation.
- Existing live-service settlement collection for the standalone MVP command.

Outputs:

- A typed in-process settlement evidence value with execution surface
  `live-service-persisted-receipt` for the canonical agent transaction.
- A secret-safe settlement evidence artifact that binds the durable escrow and
  intent digests, receipt hash, task/transaction/terms facts, fee, balances,
  signature, commitment, and authoritative RPC digest.
- A report and standalone verifier that reject override-only canonical success.

## Boundaries And Non-Goals

- Do not add dependencies, public CLI flags, chains, settlement architecture,
  or a second transfer.
- Do not persist or copy private keys, signed transaction JSON, credentials, or
  the complete service state into proof artifacts.
- Keep the command adapter available only as an explicit external test
  compatibility boundary; it cannot satisfy required-devnet report success.
- Do not change Pi/MCP tool authority or actor orchestration in this issue.
- Do not claim production, mainnet, custody, generalized exchange, or economic
  value.

## Design

1. Parse the durable service state with `serde_json`, identify the escrow and
   matching confirmed settlement intent, and validate release state,
   task/transaction/terms binding, signature, receipt hash, network,
   commitment, recipient, amount, and one submission attempt.
2. Validate all three actor artifacts through the existing actor verifier and
   require their shared public facts to equal the durable service record.
3. Independently confirm that signature through Solana devnet RPC, derive exact
   payer/recipient deltas and the transaction fee, and require fee-aware payer
   movement plus exact recipient movement.
4. Pass the resulting typed evidence directly into the existing MVP runner.
   Bind it to the runner-created live-task binding before report generation.
5. Extend the secret-safe settlement artifact and claim checks with durable
   state/intent digests, receipt hash, service task/transaction/terms facts,
   and fee. Never publish signed transaction JSON.
6. Reject `command-override` in required-devnet report verification while
   preserving isolated adapter tests.

## Failure Modes

- Persisted state, escrow, task, or settlement intent is absent or malformed.
- Escrow is not released or intent is not confirmed.
- State, actor, binding, configuration, or RPC facts disagree.
- Receipt hash or persisted signature differs from the finalized signature.
- Submission attempt count is not exactly one.
- Recipient movement is not the configured amount, payer movement is not
  amount plus fee, or RPC reports an error/non-finalized commitment.
- Durable-state or intent digest is missing, malformed, or inconsistent across
  claim, evidence artifact, and settlement log.
- Override, dry-run, placeholder, copied, or path-escaped evidence attempts to
  satisfy canonical success.

## Error Semantics

- Collection failures return `AGENT_TRANSACTION_SETTLEMENT_INVALID` with a
  bounded reason and produce canonical `NO-GO` through the existing entrypoint.
- Report/evidence mismatches return `SETTLEMENT_EVIDENCE_INVALID`.
- Funding/binding mismatches remain fail-closed and retain their existing typed
  verifier prefixes.
- No fallback from direct persisted receipt collection to command override is
  allowed.

## Acceptance Criteria

- [ ] Canonical agent-transaction `GO` reports
  `execution_surface: live-service-persisted-receipt`.
- [ ] The canonical supervisor does not execute `cat` or another command to
  inject settlement evidence.
- [ ] Durable escrow and confirmed intent fields match task, transaction,
  terms, payer/recipient, amount, signature, commitment, and actor projections.
- [ ] Evidence records exact recipient movement and payer movement equal to
  amount plus the RPC-reported fee.
- [ ] Evidence exposes only digests and safe public receipt fields, not signed
  transaction JSON, key paths, or credentials.
- [ ] Required-devnet verification rejects `command-override`, missing state,
  mismatched state, dry-run, placeholder, copied, and path-escaped evidence.
- [ ] Retry/restart proof reuses the persisted settlement and submits no second
  transfer.
- [ ] The canonical demo, standalone verifier, and independent Solana RPC
  confirmation agree on one finalized transaction.

## Files To Touch

- `crates/kamn-e2e-harness/src/agent_transaction_devnet_evidence.rs`
- `crates/kamn-e2e-harness/src/agent_transaction_finalize.rs`
- `crates/kamn-e2e-harness/src/agent_transaction_*tests*.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/devnet_settlement*.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/runner*.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/settlement_evidence_artifact.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/*settlement*verify.rs`
- Focused fixtures/contracts that encode canonical settlement evidence.
- `fixtures/ci/test_file_size_policy_baseline.env` if new bounded tests require
  a baseline increment.
- Evaluator documentation only where the old surface is described.

## Test Plan

### RED

- Require direct typed evidence in canonical finalization and reject command
  execution as the settlement handoff.
- Require durable escrow/intent reconciliation and negative cases for missing,
  mismatched, non-confirmed, duplicate-attempt, and secret-bearing state.
- Require fee-aware balance reconciliation and all three actor projections.
- Require the verifier to reject `command-override` and missing provenance
  fields for required-devnet success.

### GREEN

Implement the minimum direct collector, typed runner handoff, artifact fields,
and fail-closed verifier checks needed to pass the RED contracts.

### REFACTOR

Keep state parsing, RPC reconciliation, runner injection, and artifact
verification single-purpose. Keep files at or below 200 lines and functions at
or below 25 lines; centralize repeated field checks.

### INTEGRATION

```bash
cargo test -p kamn-e2e-harness agent_transaction
cargo test -p kamn-e2e-harness settlement_evidence
cargo test -p kamn-e2e-harness --test mvp_demo_live_task_settlement_binding_contract
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
make check
make test
make pre-push
make demo-agent-transaction
cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json
```

## Rollback

- Revert the issue commit range if direct collection cannot preserve the
  canonical demo and verifier.
- Keep the prior command adapter noncanonical for isolated compatibility tests;
  never restore it as a required-devnet success fallback.
- Preserve privacy restrictions, idempotency, finalized devnet evidence, and
  fail-closed verification during rollback.
