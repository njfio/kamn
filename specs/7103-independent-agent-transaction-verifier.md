# Issue 7103: Independent Agent Transaction Verifier

## Objective

Make the canonical verifier independently prove one completed agent transaction
from the persisted proof bundle after the KAMN node and all Pi agents have
exited. The verifier must reject copied, inconsistent, downgraded, incomplete,
or tampered evidence with stable reason codes and must not trust report prose or
process-local state.

## Inputs And Outputs

Input:

- `verify-mvp-demo --report <run>/proof/report.json`.
- The report's run-relative proof, receipt, projection, authorization, and
  settlement artifacts.
- A persisted authoritative Solana RPC response artifact for offline
  verification. Live RPC re-query is optional and is not required for the
  canonical offline command.

Output:

- Exit success and a machine-readable `PASS` result only when every required
  agent-transaction invariant is recomputed from the proof bundle.
- Exit failure with one stable reason code for the first invalid invariant.
- `report.md` containing the exact Solana Explorer devnet transaction URL.

## Boundaries And Non-Goals

- Verification must work with the proof bundle after all demo processes exit.
- Verification must not read node memory, agent process state, keypairs,
  credentials, or mutable files outside the run directory.
- The report is an index and claim envelope, not an authority for copied facts.
- The persisted Solana RPC response is the authoritative offline chain boundary;
  its digest and transaction facts must be independently checked against the
  task-bound settlement artifacts.
- This issue does not add a production indexer, mainnet support, custody
  monitoring, generalized explorer integration, or settlement retry behavior.
- This issue does not change public service API contracts or database schemas.

## Verification Contract

The verifier must independently establish:

1. All report-referenced artifacts exist under the same run directory.
2. Every artifact digest and every runtime receipt digest recomputes correctly.
3. Agent A, B, and C have distinct key-bound identities and actor roles.
4. Authorization decisions bind the verified actor, action, resource, decision,
   and reason to the corresponding runtime receipt.
5. Runtime operations are complete, unique, and in canonical order.
6. Transaction, task, escrow, terms, actors, amount, network, and settlement
   facts agree across receipts and projections.
7. Participant projections contain the required participant-only field while
   the verifier projection uses the restricted allowlist and contains no private
   field.
8. The settlement signature, recipient, amount, network, commitment/finality,
   and payer/recipient balance deltas agree with the persisted authoritative RPC
   response and the task-bound escrow evidence.
9. Local-only, dry-run, placeholder, ambiguous, missing-actor, or synthetic
   required evidence cannot satisfy agent-transaction success.
10. The human report's Solana Explorer URL is derived from the verified devnet
    signature and equals
    `https://explorer.solana.com/tx/<signature>?cluster=devnet`.

## Failure Modes And Error Semantics

The command fails closed and returns one stable machine-readable code:

- `PROOF_ARTIFACT_PATH_INVALID`: referenced artifact escapes or belongs to a
  different run directory.
- `PROOF_ARTIFACT_MISSING`: a required artifact is absent.
- `PROOF_ARTIFACT_TAMPERED`: an artifact or receipt digest does not recompute.
- `AGENT_IDENTITY_INVALID`: actor identities are missing, duplicated, or not
  key-bound as claimed.
- `AUTHORIZATION_EVIDENCE_INVALID`: role, action, actor, resource, decision, or
  reason does not match its runtime receipt.
- `RECEIPT_CHAIN_INVALID`: required operations are missing, duplicated,
  reordered, synthetic, or cross-transaction.
- `TRANSACTION_AGREEMENT_INVALID`: task, escrow, terms, actor, amount, or network
  facts disagree.
- `PROJECTION_SCOPE_INVALID`: participant membership, restricted allowlist,
  shared commitment, or private-field boundary is invalid.
- `SETTLEMENT_EVIDENCE_INVALID`: signature, recipient, amount, network,
  commitment, finality, balance movement, or authoritative RPC evidence is
  absent or inconsistent.
- `AGENT_TRANSACTION_CLAIM_INVALID`: success depends on local-only, dry-run,
  placeholder, ambiguous, synthetic, or missing actor evidence.
- `EXPLORER_LINK_INVALID`: the human report link is absent or differs from the
  verified signature-derived devnet URL.

Interior helpers return `Result<T, String>` using these codes. The CLI entrypoint
prints the failure once and exits non-zero. No failure is downgraded or silently
replaced with copied report data.

## Acceptance Criteria

- [x] The canonical verifier recomputes every required artifact and receipt
      digest from files in the proof bundle.
- [x] Identity, authorization, operation order, agreement, projections, and
      settlement evidence are checked from authoritative artifacts.
- [x] Offline settlement verification validates a digest-bound authoritative
      Solana RPC response, recipient, amount, network, signature, finalized
      commitment, and exact payer/recipient balance movement.
- [x] `report.md` contains the exact signature-derived Solana Explorer devnet
      transaction link.
- [x] Verification passes after node and agent processes have exited and after
      process-local state outside the proof bundle is unavailable.
- [x] Tampering with signature, recipient, amount, commitment, role, operation
      order, public projection, actor receipt, or explorer link fails with a
      stable reason code.
- [x] A signature copied from another escrow fails.
- [x] Local-only, dry-run, placeholder, ambiguous, synthetic, and missing-actor
      evidence cannot produce agent-transaction PASS.
- [x] Existing report schema verification remains compatible; the stricter
      agent-transaction success contract is fail-closed.
- [x] Formatting, strict clippy, the verifier negative matrix, and `make check`
      pass.

## Files To Touch

Expected implementation surface:

- `crates/kamn-e2e-harness/src/mvp_demo/verify.rs`
- New focused verifier modules under
  `crates/kamn-e2e-harness/src/mvp_demo/`
- `crates/kamn-e2e-harness/src/mvp_demo/report_markdown.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/mod.rs`
- Focused verifier contract tests under `crates/kamn-e2e-harness/tests/`
- Test support fixtures under `crates/kamn-e2e-harness/tests/support/`
- `docs/validation/mvp-evaluator-demo.md`

No service API schema, persistent node-state schema, SDK API, workflow, or shell
surface is expected to change.

## Test Plan

RED tests first:

1. A valid canonical proof bundle passes with all processes absent.
2. A signature copied from another settlement artifact fails.
3. Mutating recipient, amount, finality/commitment, balance delta, role,
   operation order, public projection, or explorer link fails with the specified
   stable category code.
4. Removing one actor/runtime receipt while retaining transcript prose fails.
5. Substituting local-only, dry-run, placeholder, or ambiguous settlement
   evidence fails.
6. Moving or referencing an artifact outside the run directory fails.

Verification commands:

```bash
cargo test -p kamn-e2e-harness independent_agent_transaction_verifier -- --nocapture
cargo test -p kamn-e2e-harness mvp_demo_runtime_receipt_chain -- --nocapture
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
make check
cargo run -p kamn-e2e-harness -- \
  verify-mvp-demo --report .kamn/demo/latest/proof/report.json
```

Completion evidence must include the passing negative matrix and one standalone
verification executed after confirming the demo's node and Pi child processes
are no longer running.

## Completion Evidence

- Canonical devnet run: `run-67664-1783861118291`, finalized signature
  `aGdmjmRELAPZ8k7Fe6Gmn8Ajy781Ru8QVwYSAGaNLxoKCGmsSSZDBBi4YwbetdZ6UxhFEJgMk6y4EYsN7oGKGBi`.
- The run moved `1000000` lamports from the configured payer to recipient and
  retained the finalized raw Solana confirmation response in the proof bundle.
- Standalone verification passed for both `.kamn/demo/latest/proof/report.json`
  and the immutable run report after all KAMN and Pi processes exited.
- The focused negative matrix passed 42 tests; `cargo fmt --check`, strict
  workspace clippy, and `make check` passed on the final implementation.
- The broader `make test` remains blocked by the pre-existing
  `current_branch_head_restores_ratio_compliance` branch-history baseline, which
  reproduces on `origin/main` and is outside this issue's verifier scope.
