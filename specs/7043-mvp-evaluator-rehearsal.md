# 7043 MVP Evaluator Rehearsal

## Objective

Validate KAMN from a clean evaluator perspective on current `main`: follow the
root README and linked evaluator runbook, run the canonical MVP demo, verify the
generated report, run the funded Solana devnet-required path when local
non-committed devnet config is available, and record agent-harness feasibility
for goose and pi without weakening proof boundaries.

## Inputs/Outputs

Inputs:
- Current `main` checkout and clean external evaluator worktree
- Root `README.md`
- `docs/validation/mvp-evaluator-demo.md`
- Local non-committed `.kamn/devnet/mvp-demo-devnet.env` when available
- Current official goose/pi documentation and local CLI availability

Outputs:
- Fresh-checkout local-only MVP demo evidence
- Fresh-checkout devnet-required MVP demo evidence or explicit `NO-GO` blocker
- Goose/pi harness findings or exact blocker evidence
- `docs/validation/2026-07-07-mvp-evaluator-rehearsal.md`
- A docs contract test protecting the required evidence report shape

## Boundaries/Non-goals

- Do not change MVP demo behavior unless rehearsal exposes a real defect.
- Do not commit devnet keypairs, private RPC credentials, `.kamn/` proof
  artifacts, generated package metadata, or fresh-checkout build outputs.
- Do not claim production readiness, mainnet settlement, generalized exchange,
  or real economic value.
- Do not weaken tests, lint, clippy, formatting, verifier semantics, proof
  semantics, or claim boundaries.
- Keep README human-first; put rehearsal evidence and deeper details in
  validation docs.

## Deferred to Implementation

- Resolve whether goose and/or pi can run locally with existing machine auth
  without new committed secrets.
- Resolve whether a safe `.env.example` is needed after the clean evaluator
  rehearsal. Add one only if a real evaluator gap appears.

## Failure Modes

- Clean worktree local demo fails from README/runbook instructions.
- Report verifier rejects the generated local-only report.
- Devnet-required run silently downgrades settlement to local-only instead of
  explicit `GO` devnet-backed or honest `NO-GO`.
- Proof report/logs leak private key contents, local keypair file contents, or
  private credentials.
- Agent harness setup requires unclear auth, secrets, or unsupported local
  assumptions.
- Evidence doc omits run ids, commands, report paths, transaction signature or
  blocker, claim-boundary inspection, or harness findings.

## Acceptance Criteria

- [ ] Clean external worktree is created from current `main` and recorded.
- [ ] Local-only `make demo-mvp` succeeds in that clean worktree.
- [ ] Local-only report verifier returns `PASS`.
- [ ] Funded Solana devnet-required run returns `GO` with `devnet-backed`
  settlement evidence and verifier `PASS`, or records exact `NO-GO` blocker.
- [ ] Proof artifacts are inspected for claim-boundary clarity and absence of
  secret/keypair content.
- [ ] Goose and pi are researched from current official sources and locally
  attempted where feasible without committed secrets.
- [ ] Blockers for goose/pi are documented with exact evidence if either harness
  cannot complete the evaluator path.
- [ ] Evidence report is protected by a docs contract test.
- [ ] Local gates pass before PR.

## Files to Touch

Likely:
- `docs/validation/2026-07-07-mvp-evaluator-rehearsal.md`
- `crates/kamn-e2e-harness/tests/mvp_evaluator_rehearsal_docs_contract.rs`

Only if a real evaluator gap is found:
- `docs/validation/mvp-evaluator-demo.md`
- `.env.example` or a narrowly scoped devnet env example

## Error Semantics

- Rehearsal commands must fail loudly in the evidence report; do not reinterpret
  failures as success.
- Devnet-required `NO-GO` is acceptable only when it is explicit and backed by
  real blocker evidence.
- Any settlement or asset-movement success must remain `devnet-backed`.

## Test Plan

Execution note: characterize first for external worktree/demo behavior, then use
test-first discipline for the committed evidence report.

Characterization:
- `git fetch --all --prune`
- Create a clean external worktree from `origin/main`.
- Run `make demo-mvp` from the clean worktree.
- Run `cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json`.
- Run funded devnet-required `make demo-mvp` from the clean worktree if local
  non-committed devnet config is available.
- Confirm Solana transaction status with CLI when devnet run succeeds.

Red:
- Add a docs contract test requiring the evidence report, command evidence,
  run ids, claim-boundary findings, goose/pi findings, and secret-scan outcome.
- Confirm it fails before the evidence report exists.

Green:
- Add the evidence report with exact commands, run ids, report paths, devnet
  transaction or blocker, agent-harness findings, and remaining risks.

Verification:
- `cargo fmt --check`
- `cargo test -p kamn-e2e-harness --test mvp_evaluator_rehearsal_docs_contract -- --nocapture`
- `cargo test -p kamn-e2e-harness --test readme_mvp_front_door_contract --test mvp_demo_command_contract --test mvp_demo_claim_contract -- --nocapture`
- `make check`
- `make demo-mvp`
- `cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json`
