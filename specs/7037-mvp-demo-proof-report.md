# 7037-mvp-demo-proof-report

## Objective
Create one evaluator-facing MVP demo command that packages KAMN's existing bounded runtime proof slices into a coherent local demo artifact. The command must generate a machine-readable proof report and a human-readable report, and the verifier must fail closed on malformed, downgraded, placeholder, or overclaimed proof reports.

The MVP is not production readiness. The MVP is a locally runnable and locally provable KAMN story with honest claim boundaries.

## Inputs/Outputs
### Inputs
- Current `main` KAMN workspace after PR `#7022`.
- Existing KAMN runtime, service API, SDK, CLI/MCP, audit, websocket, relay, and `kamn-e2e-harness` surfaces.
- Optional Solana devnet configuration:
  - `KAMN_MVP_DEVNET_MODE=required`
  - `KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com`
  - optional future keypair/funding inputs if needed by the current devnet-backed asset movement path.

### Outputs
- `make demo-mvp`
- `cargo run -p kamn-e2e-harness -- demo-mvp`
- `cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json`
- `.kamn/demo/<run-id>/proof/report.json`
- `.kamn/demo/<run-id>/proof/report.md`
- `.kamn/demo/latest/proof/report.json`
- `.kamn/demo/latest/proof/report.md`
- Evaluator runbook documenting the command, artifacts, claim boundaries, and devnet `NO-GO` behavior.

## Boundaries/Non-goals
- Do not claim production readiness.
- Do not claim mainnet support.
- Do not claim generalized exchange or broad multi-chain settlement.
- Do not claim consensus, broad bridge finality, or arbitrary partition tolerance.
- Do not replace existing runtime, harness, service API, SDK, CLI, or MCP architecture.
- Do not add a UI or dashboard.
- Do not count in-memory-only settlement or asset movement as MVP settlement success.
- Do not silently downgrade devnet-required settlement failure to local-only success.
- Do not weaken tests, formatting, clippy, proof semantics, local gates, or claim boundaries.

## Failure modes
- `make demo-mvp` is missing or not wired to a real workspace command.
- The demo writes no run directory or no `.kamn/demo/latest` proof path.
- The report omits required claim labels or emits unknown labels.
- A required MVP success claim is labeled `dry-run` or `placeholder`.
- A settlement, escrow, exchange, transfer, lamports, asset, or value-movement claim is not labeled `devnet-backed`.
- Devnet-required mode cannot fund, submit, confirm, or prove balance movement and still exits as a silent local-only pass.
- The verifier accepts malformed JSON, missing required sections, downgraded labels, placeholder details, or settlement claims without devnet evidence.
- The human-readable report disagrees with the JSON report.
- The runbook overstates the proof beyond local MVP demo readiness.

## Acceptance criteria (testable booleans)
- [ ] `make demo-mvp` exists.
- [ ] `make demo-mvp` creates `.kamn/demo/<run-id>/` and `.kamn/demo/latest`.
- [ ] The demo writes `.kamn/demo/latest/proof/report.json`.
- [ ] The demo writes `.kamn/demo/latest/proof/report.md`.
- [ ] The JSON report includes a claim matrix.
- [ ] Every claim label is one of `real`, `devnet-backed`, `local-only`, `dry-run`, `placeholder`, or `roadmap`.
- [ ] Required MVP success claims reject `dry-run` and `placeholder`.
- [ ] Claims involving exchange, escrow, settlement, transfer, lamports, asset movement, or value movement require `devnet-backed`.
- [ ] The report includes local runtime startup, authenticated Alice/Bob identities, signed flow, durable state, relay/projection, websocket/event visibility, and audit/proof export.
- [ ] The verifier rejects malformed, missing, downgraded, placeholder, and settlement-without-devnet reports.
- [ ] The verifier accepts a valid local-only runtime report without settlement success claims.
- [ ] The verifier accepts a valid devnet-backed settlement report.
- [ ] `KAMN_MVP_DEVNET_MODE=required KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com make demo-mvp` either writes devnet-backed settlement evidence or explicit `NO-GO` evidence.
- [ ] The evaluator runbook documents local-only, devnet-backed, dry-run, placeholder, roadmap, and `NO-GO` interpretation.

## Files to touch
- `specs/7037-mvp-demo-proof-report.md`
- `Makefile`
- `crates/kamn-e2e-harness/src/main.rs`
- `crates/kamn-e2e-harness/src/lib.rs`
- new bounded modules under `crates/kamn-e2e-harness/src/` for MVP demo report generation and verification
- targeted tests under `crates/kamn-e2e-harness/tests/`
- an evaluator runbook under `docs/`
- optionally `README.md` if command discovery needs a concise pointer

## Error semantics
- `demo-mvp` must hard-fail on local filesystem/report write errors.
- `demo-mvp` must not report MVP success if required local runtime proof artifacts cannot be generated.
- `demo-mvp` in devnet-required mode must write explicit `NO-GO` evidence when devnet evidence is unavailable.
- `verify-mvp-demo` must return non-zero on malformed JSON, missing required sections, unknown labels, placeholder required claims, dry-run required claims, or settlement/value claims without devnet-backed evidence.
- `verify-mvp-demo` may accept local-only runtime reports only when no settlement/value-movement success claim is present.
- Error messages must name the rejected contract so failures are actionable.

## Test plan
1. Phase 3 red tests:
   - add command parser/contract tests for `demo-mvp` and `verify-mvp-demo`
   - add report schema tests for required sections, labels, and artifact paths
   - add negative verifier fixtures for malformed, missing, downgraded, placeholder, and settlement-without-devnet reports
   - add positive verifier fixtures for local-only runtime proof and devnet-backed settlement proof
   - add Makefile command-surface test for `make demo-mvp`
2. Phase 4 green implementation:
   - add a Rust-owned demo report writer under `kamn-e2e-harness`
   - create run directories and a `.kamn/demo/latest` proof path
   - render deterministic JSON and Markdown reports
   - wire `verify-mvp-demo --report`
   - wire `make demo-mvp`
3. Phase 5 refactor:
   - split modules so files remain bounded and functions stay single-purpose
   - remove duplication between report generation and verifier checks
   - keep claim taxonomy centralized
4. Phase 6 integration:
   - run the new command through Makefile
   - verify generated report with the new verifier command
   - document evaluator usage and claim boundaries
5. Final checks:
   - `cargo test -p kamn-e2e-harness --test mvp_demo_claim_contract -- --nocapture`
   - `cargo test -p kamn-e2e-harness --test mvp_demo_command_contract -- --nocapture`
   - `cargo fmt --check`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `make check`
   - `make demo-mvp`
   - `cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json`
   - `KAMN_MVP_DEVNET_MODE=required KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com make demo-mvp`
   - `make pre-push` before publishing unless a real external/toolchain blocker is documented.

## Deviations / Final evidence
- Implementation uses `kamn-e2e-harness` as the canonical demo/report/verifier entrypoint and `make demo-mvp` as the evaluator command.
- `make demo-mvp` uses `target/mvp-demo-proof` internally to avoid the known stale/default-target cargo wedge observed during local verification.
- The default local run produces `status=GO` with local-only runtime/auth/message/state/relay/websocket/audit claims and no settlement success claim.
- Devnet-required mode currently produces explicit `status=NO-GO` evidence with `no_go.reason=devnet_keypair_not_configured` because no funded Solana devnet settlement keypair is configured.
- Default-target `make check` wedged in stale `kamn_core` clippy workers at 0% CPU; the same gate was rerun and passed as `CARGO_TARGET_DIR=target/mvp-workspace-check make check`.

Verification evidence:
- `CARGO_TARGET_DIR=target/mvp-demo-contract cargo test -p kamn-e2e-harness --test mvp_demo_claim_contract -- --nocapture` passed, 10 tests.
- `CARGO_TARGET_DIR=target/mvp-demo-command cargo test -p kamn-e2e-harness --test mvp_demo_command_contract -- --nocapture` passed, 5 tests.
- `cargo fmt --check` passed.
- `cargo clippy -p kamn-e2e-harness --all-targets --all-features -- -D warnings` passed.
- `CARGO_TARGET_DIR=target/mvp-workspace-check make check` passed.
- `make demo-mvp` passed and generated `.kamn/demo/latest/proof/report.json` plus `.kamn/demo/latest/proof/report.md`.
- `CARGO_TARGET_DIR=target/mvp-demo-proof cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json` passed.
- `KAMN_MVP_DEVNET_MODE=required KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com make demo-mvp` passed process execution and generated explicit devnet `NO-GO` evidence.
