# Issue #7135: Independently Verify Service Receipt Authority

## Objective

Make the Rust evaluator independently recompute the versioned service receipt-chain
commitment from durable service state after every Pi and MCP child exits. Require the
canonical demo and standalone verifier to bind that commitment to all three v2 actor
artifacts and the existing settlement proof without accepting Pi-local evidence as an
authority substitute.

## Inputs And Outputs

### Inputs

- The finalized `service-api-state.json` produced by the live service.
- Three `kamn.mvp.pi-transaction-actor.v2` artifacts emitted by persistent,
  role-scoped authenticated MCP sessions.
- Participant-private projections for agents A and B and the restricted-public
  projection for agent C.
- The existing #7125 settlement receipt and authoritative Solana RPC artifact.

### Outputs

- A Rust-computed `kamn.service.receipt-chain.v1` commitment derived from durable
  authorization, task, escrow, and confirmed-settlement records.
- Settlement evidence that binds the durable receipt-chain commitment to the one
  finalized Solana transfer.
- Bundled v2 actor evidence whose three actors share the same receipt-chain and public
  commitments while retaining role-private receipt disclosure.
- A standalone verification result that rejects any proof satisfiable by client-local
  hashes or v1 actor receipts alone.

## Boundaries And Non-Goals

- Do not change service receipt generation, projection semantics, Pi v2 envelope
  generation, public commitment calculation, or the #7125 settlement path.
- Do not add dependencies, chains, OAuth, remote MCP transport, production custody, or
  mainnet behavior.
- Do not copy the complete durable service store into the portable proof bundle; carry
  only the independently recomputed commitment and existing allowlisted proof fields.
- Do not expose authorization, correlation, idempotency, or participant-private receipt
  details through agent C's artifact.
- Transport response digests remain provenance only and never satisfy service authority.

## Failure Modes

- V1, mixed-version, malformed, copied, missing, duplicated, reordered, replayed, or
  cross-role actor authority.
- Actor service receipts that do not exactly match the allowed role, action, resource,
  resulting state, and final participant projection.
- A durable receipt record with an invalid actor, authorization binding, action order,
  resource, state transition, idempotency key, settlement binding, or digest.
- Any actor receipt-chain or public commitment disagreement.
- Any durable commitment disagreement with the actor or settlement proof commitment.
- A path outside the expected proof root, missing artifact, malformed JSON, or digest
  mismatch.
- More than one settlement submission attempt, a changed stable receipt identity, or an
  RPC result that does not prove exactly one finalized transfer.
- MCP child replacement, non-contiguous transport request provenance, or actor process
  reuse across roles.

## Acceptance Criteria

- [x] The harness recomputes `kamn.service.receipt-chain.v1` from finalized durable state
  after all Pi and MCP children have exited.
- [x] The recomputation matches the node's canonical ordering and length-prefixed digest
  algorithm for create, accept, fund, complete, release, and confirmed settlement.
- [x] All three v2 actor artifacts bind one receipt-chain commitment and one public
  commitment; A and B expose only their exact role receipts and C exposes none.
- [x] V1, client-only, missing, copied, reordered, cross-role, replayed, path-escaped,
  malformed, and projection-mismatched evidence fails closed.
- [x] The canonical supervisor continues to use persistent role-scoped authenticated MCP
  children and cannot fall back to v1 Pi-local authority.
- [x] Retry, restart, and ambiguous-response verification prove stable service receipt
  identity, one settlement attempt, and no duplicate task, escrow, or Solana action.
- [x] The existing settlement receipt and independent Solana RPC proof remain mandatory
  and prove exactly one finalized transfer.
- [ ] `make demo-agent-transaction` and standalone proof verification use service-backed
  authority from a fresh checkout.
- [x] Tamper, crash, replay, privacy, and restart cases fail closed or pass only with the
  expected stable durable authority.
- [ ] Formatting, strict Clippy, `make check`, package-aware tests, pre-push checks,
  mutation gates, and CI evidence are recorded.

## Files To Touch

- `crates/kamn-e2e-harness/src/agent_transaction_persisted_settlement.rs`
- New focused durable receipt-chain parser and commitment modules under
  `crates/kamn-e2e-harness/src/`.
- `crates/kamn-e2e-harness/src/agent_transaction_devnet_evidence.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/pi_transaction_actor_model.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/pi_transaction_actor_verify.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/runtime_actor_bundle.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/runner.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/devnet_settlement.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/settlement_evidence_artifact.rs`
- Focused harness fixtures and tests for v2 authority, durable recomputation, portable
  proof verification, retry, privacy, and tamper behavior.

## Error Semantics

- Durable chain parsing, ordering, authorization, digest, or commitment disagreement
  returns `SERVICE_RECEIPT_CHAIN_INVALID`.
- Actor service authority or projection disagreement returns
  `PI_SERVICE_AUTHORITY_MISMATCH`.
- Actor process, child, request range, or transport hash disagreement returns
  `PI_TRANSPORT_PROVENANCE_INVALID`.
- Settlement, RPC, and duplicate-action disagreement retains
  `AGENT_TRANSACTION_SETTLEMENT_INVALID` or `SETTLEMENT_EVIDENCE_INVALID` at their
  existing boundaries.
- Filesystem and JSON failures hard-fail with the owning boundary code and preserve a
  concise cause. There is no v1 or client-local compatibility fallback.

## Test Plan

### RED

- Add Rust tests requiring v2 actors, exact role receipt sets, shared commitments,
  projection agreement, and rejection of v1/client-local authority.
- Add durable-state tests for the canonical six-entry chain and each tampered actor,
  action, order, state, authorization, idempotency, settlement, and retry field.
- Add portable proof tests requiring the independently recomputed commitment in the
  settlement artifact and matching it against the bundled actors.
- Add path escape, copied receipt, replay, privacy, restart, and ambiguous-response cases.

### GREEN

- Parse the finalized durable state into narrow typed records and reproduce the node's
  canonical authority and chain digests with the existing SHA-256 dependency.
- Migrate the Rust actor model and verifier to v2 service receipts and transport-only
  provenance.
- Bind the recomputed commitment into settlement evidence and the portable artifact, then
  require it throughout canonical and standalone verification.

### REFACTOR

- Separate durable parsing, authority digesting, chain assembly, actor verification, and
  proof binding into single-purpose modules within repository size limits.
- Remove canonical reliance on locally synthesized runtime receipt chains while retaining
  only explicitly labeled transport provenance.
- Run formatting, strict Clippy, focused tests, and the full package checks.

### INTEGRATION

- Run the real three-role workflow with persistent MCP children and finalized service
  state, then verify the portable proof after all children exit.
- Run settlement RPC reconciliation and prove one confirmed submission attempt.
- Exercise deliberate actor, durable-state, projection, path, replay, privacy, crash, and
  restart variants.
- Run fresh-checkout canonical demo and standalone verification, pre-push checks, mutation
  gates, and repository CI.

## Verification Evidence

### Local Passes

- `CARGO_BUILD_JOBS=1 cargo clippy -p kamn-e2e-harness --all-targets -- -D warnings`
- `CARGO_BUILD_JOBS=1 make check`
- Touched Rust size policy: `GO`, with no oversized touched files or functions.
- Critical-path mutation gate: `GO`; 10 of 10 mutants caught across six slices, with no
  misses, timeouts, or unviable mutants.
- Canonical post-child-exit agent transaction and standalone verifier contract.
- Focused service-authority, v2 actor, portable receipt-chain, settlement, retry/restart,
  privacy, tamper, path, and error-boundary contracts.
- Package library tests: 259 passed before integration-binary startup.

### Incomplete Local Lanes

- The package-wide test run was stopped after all 259 library tests passed because the
  macOS host stalled while starting integration binaries already covered by focused runs.
- `make ci-tools` completed multiple shell-contract families, then stopped making progress
  with no remaining child process and was terminated. It is not counted as a pass.
- Fresh-checkout `make demo-agent-transaction`, the complete `make pre-push` sequence, and
  repository CI remain pending at this closeout checkpoint.

### Deviations

- No service receipt generation, public projection, dependency, database, CI, or public
  command contract was changed.
- Runtime-actor integration fixtures were migrated from command-override settlement to the
  canonical persisted-service settlement path because command overrides are not settlement
  authority when actor evidence is present.
