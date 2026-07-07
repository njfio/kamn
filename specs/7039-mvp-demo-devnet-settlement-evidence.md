# 7039 MVP Demo Devnet Settlement Evidence

## Objective

Wire the canonical MVP demo to the existing service API live Solana settlement lane so a devnet-required demo can produce real devnet-backed settlement evidence when funded keypairs are configured.

The demo must remain honest: incomplete Solana configuration, unfunded keypairs, RPC failures, submission failures, and confirmation failures must produce explicit failure or `NO-GO` evidence rather than a local-only pass.

## Inputs/Outputs

Inputs:
- `KAMN_MVP_DEVNET_MODE=optional|required`
- `KAMN_MVP_SOLANA_RPC_URL`
- `KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL`
- `KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE`
- `KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY`
- `KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS`
- `KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT`

Outputs:
- `.kamn/demo/<run-id>/proof/report.json`
- `.kamn/demo/<run-id>/proof/report.md`
- `.kamn/demo/latest/proof/report.json`
- `.kamn/demo/latest/proof/report.md`
- a `devnet-backed` settlement claim only when a real Solana devnet transfer was submitted and confirmed

## Boundaries/Non-goals

- Do not add new settlement architecture.
- Do not use fake or in-memory-only value movement for a settlement success claim.
- Do not make mainnet, production custody, broad bridge finality, or production readiness claims.
- Do not write private key material into tracked files, reports, or logs.
- Do not weaken local-only MVP proof claims, verifier semantics, formatting, clippy, or tests.

## Failure Modes

- Unsupported `KAMN_MVP_DEVNET_MODE` fails before proof output.
- Devnet-required mode with missing or partial Solana settlement env emits explicit `NO-GO`.
- Invalid keypair file, invalid recipient pubkey, invalid lamports, or invalid commitment emits explicit `NO-GO`.
- RPC blockhash, transaction submission, confirmation, or balance lookup failure emits explicit `NO-GO`.
- A report with placeholder settlement fields, missing signature fields, or non-devnet labels is rejected by the verifier.
- Releasing an already-released escrow must return the persisted signature and must not submit a duplicate transfer.

## Acceptance Criteria

- [ ] `KAMN_MVP_DEVNET_MODE=optional make demo-mvp` keeps existing local-only GO behavior.
- [ ] `KAMN_MVP_DEVNET_MODE=required` without complete Solana settlement env writes explicit devnet `NO-GO`.
- [ ] `KAMN_MVP_DEVNET_MODE=required` with funded devnet keypairs submits one Solana devnet transfer through the service API escrow release path.
- [ ] The report contains `devnet-backed` PASS evidence with `network`, `rpc_url`, `payer_pubkey`, `recipient_pubkey`, `lamports`, `settlement_tx_signature`, `settlement_commitment`, balance-before/after fields, and `persisted_settlement_tx_signature`.
- [ ] The verifier accepts the real devnet-backed evidence shape.
- [ ] The verifier rejects devnet-backed reports missing required settlement evidence.
- [ ] No private key contents are exposed in generated report fields.

## Files to Touch

Likely:
- `crates/kamn-e2e-harness/src/mvp_demo/runner.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/report.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/verify.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_claim_contract.rs`
- `docs/validation/mvp-evaluator-demo.md`

Only if necessary:
- `crates/kamn-e2e-harness/src/mvp_demo/mod.rs`
- `crates/kamn-e2e-harness/src/lib.rs`

## Error Semantics

- The demo command may return process success with a `NO-GO` report for expected devnet unavailability, matching the existing MVP report contract.
- Unexpected local proof failures must return an error.
- Settlement success requires real devnet transaction evidence and persisted KAMN state evidence.
- Reports must distinguish `real`, `local-only`, `devnet-backed`, `dry-run`, `placeholder`, and `roadmap` claims.

## Test Plan

Red tests first:
- report generation test for configured devnet settlement evidence producing a `devnet-backed` PASS claim
- verifier negative test rejecting missing settlement signature or persisted signature
- verifier positive test accepting a devnet-backed settlement fixture
- optional command-config test covering settlement env parsing boundaries

Green implementation:
- add a minimal devnet settlement artifact collector in the MVP demo harness that reuses the service API live Solana release path semantics
- extend report rendering with optional settlement evidence
- extend verifier requirements for devnet-backed settlement claims

Verification:
- `cargo fmt --check`
- `cargo test -p kamn-e2e-harness --test mvp_demo_claim_contract -- --nocapture`
- `cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json`
- `make demo-mvp`
- funded local devnet run with `.kamn/devnet/mvp-demo-devnet.env`
