---
date: 2026-06-26
topic: kamn-mvp-demo-readiness
origin: docs/brainstorms/2026-06-26-kamn-forward-strategy-requirements.md
status: execution-ready
artifact_status: historical
current_status: superseded
---

# KAMN MVP Demo Readiness Plan

> Historical decision artifact. The July 10 agent-transaction plan supersedes
> this execution plan. It is not production or mainnet evidence; current
> settlement claims remain limited to Solana devnet.

## Executive Decision

Move KAMN from "bounded proof slices exist" to "MVP demo is real, reproducible, and locally provable" by consolidating the existing service API, durable state, relay, websocket, audit, and Solana devnet asset-movement slices into one evaluator-facing demo path.

The MVP is not broad production readiness. The MVP is one coherent local KAMN story:

1. A fresh checkout can run one canonical demo command or a small command sequence.
2. The local KAMN runtime starts real local services.
3. Two authenticated agent identities execute a message/task flow.
4. Durable state, relay/projection, websocket/event visibility, and audit/proof export are all exercised from the same run.
5. Any escrow, settlement, exchange, or asset-movement claim is backed by Solana devnet transaction evidence, never by fake in-memory settlement.
6. The final report labels each claim as `real`, `devnet-backed`, `local-only`, `dry-run`, `placeholder`, or `roadmap`.

Recommended canonical outcome:

```bash
make demo-mvp
```

The implementation should allow a preflight/configured form for evaluator runs:

```bash
KAMN_MVP_DEVNET_MODE=required \
KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com \
make demo-mvp
```

`make demo-mvp` may generate ephemeral devnet key material and request faucet funds when no explicit devnet keypair is supplied. For reliable recorded demos, it must also support a pre-funded devnet keypair file override. If devnet funding, transaction submission, confirmation, or balance proof fails, the demo must produce `NO-GO` evidence, not silently downgrade to local-only or dry-run settlement.

## Source Requirements Carried Forward

Source: `docs/brainstorms/2026-06-26-kamn-forward-strategy-requirements.md`

Hard requirements carried into this plan:

- Local quality gates are currently red and must be fixed before feature expansion.
- Do not weaken formatting, strict clippy, tests, or proof semantics.
- Prefer consolidating working surfaces over adding new architecture.
- Prioritize proof depth over new specs, governance, or documentation breadth.
- Explicitly separate `real`, `devnet-backed`, `local-only`, `dry-run`, `placeholder`, and `roadmap`.
- Use devnet-backed execution/evidence for any exchange, escrow, settlement, or asset-movement claim.
- Keep the final result credible for a technical evaluator, not only CI.

Resolved planning decisions from the brainstorm:

- Canonical proof path: a new `make demo-mvp` wrapper over a Rust-owned demo/proof runner, not the current localhost-only SDK demo.
- Strict clippy strategy: one gate-recovery issue first, split by crate/module only if the first issue becomes too large to review safely.
- Primary product proof path: authenticated agent task/message flow with devnet-backed escrow release/asset movement as the settlement proof.
- CI lane shape: PR proves contract/schema/local deterministic slices; scheduled/manual proves live devnet and full evaluator run.

## Branch And Base State

Planning base after refresh:

- Local branch: `main`
- Upstream tracking: `origin/main`
- Latest integrated non-dependency product base: `origin/main`
- Open PRs found during refresh: Dependabot-only Cargo updates:
  - `#7015` `dependabot/cargo/libp2p-gossipsub-0.49.4`
  - `#7017` `dependabot/cargo/rand-0.8.6`
  - `#7018` `dependabot/cargo/fuzz/rand-0.8.6`
  - `#7019` `dependabot/cargo/rustls-webpki-0.103.13`
- Newer non-dependency branch scan: `origin/6880-verify-task-operations` exists but is stale relative to `main`.

Decision: plan against `origin/main`; do not switch to a dependency branch for product/MVP work.

Current local gate evidence:

- `cargo fmt --check` exits `1` with large formatting drift across workspace files.
- `cargo clippy --workspace --all-targets --all-features --message-format short -- -D warnings` exits `101` in `kamn-core`; representative failures include unused imports, `unused_mut`, `type_complexity`, `ptr_arg`, `manual_is_multiple_of`, wildcard-or-pattern, needless question mark, too many arguments, test `unwrap`, missing docs, module inception, and redundant closures.
- Because `Makefile` defines `make check` as `cargo fmt --check` plus strict workspace clippy, `make check` is currently red.

## Existing Surfaces To Consolidate

Use these existing surfaces. Avoid a parallel demo architecture unless the existing surface blocks proof clarity.

### Local Runtime And Service API

Relevant files:

- `crates/kamn-node/src/main.rs`
- `crates/kamn-node/src/service_api_endpoint/server.rs`
- `crates/kamn-node/src/service_api_endpoint/state_io.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl/http_routes.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl/http_routes/mutations/update_routes/message_routes.rs`
- `crates/kamn-node/src/service_api_endpoint/websocket.rs`
- `crates/kamn-sdk/src/live.rs`
- `crates/kamn-sdk/src/service_client.rs`

What is real now:

- Service API runtime starts from `kamn-node`.
- Auth public key, TLS, state file, relay spool, live Solana bridge, live settlement, replay guard, anti-spam, request budget, and websocket fanout are initialized at startup.
- State file defaults can be overridden; JSON persistence is atomic; `.sqlite`, `.sqlite3`, and `.db` state files route through SQLite storage.
- Relay spool is durable NDJSON and can be projected into relayed message state.
- Message send persists state, appends relay entries, and publishes websocket events.
- Websocket fanout publishes message, channel, task, and bridge lifecycle events with sequence numbers.

### Existing Proof Slices

Relevant docs:

- `docs/validation/current-proven-runtime-slices.md`
- `docs/validation/working-vertical-slice.md`
- `docs/validation/durable-cross-node-relay-slice.md`
- `docs/validation/restart-persistence-slice.md`
- `docs/validation/live-solana-devnet-proof-slice.md`
- `docs/validation/live-solana-bridge-dispatch-slice.md`
- `docs/validation/live-solana-bridge-websocket-slice.md`
- `docs/validation/escrow-settlement-slice.md`
- `docs/validation/external-chain-backed-settlement-slice.md`
- `docs/validation/solana-devnet-asset-movement-slice.md`

What is real now:

- Working vertical slice proves two identities, service API message delivery, task lifecycle, and audit export.
- Durable relay slice proves relay spool preservation, later projection, and recipient-visible delivered state after fresh boot.
- Restart persistence slice proves message, task, escrow, directory, relayed, and delivered state continuity.
- Live Solana devnet proof captures live JSON-RPC health/version/slot evidence and normalizes finality labels.
- Live Solana bridge websocket slice proves live-backed bridge evidence reaches `/v1/events/ws`.
- Solana devnet asset-movement slice proves a bounded runtime path for real Solana devnet transfer submission and persisted `settlement_tx_signature`, with an ignored live proof for direct devnet execution.

What is not real enough:

- The current `make demo` is a localhost signed SDK exchange, not the full product story.
- `crates/kamn-sdk/examples/local_e2e_demo.rs` uses `InMemoryKamnClient`; it must not be used for MVP settlement/asset-movement claims.
- `crates/kamn-e2e-harness/src/run_contract/orchestration/phase_model/steps/infra_deploy.rs` still emits `deterministic placeholder` details for infra/deploy steps. These placeholders cannot count as MVP runtime proof.
- Current proof is fragmented across docs, tests, scripts, ignored live tests, local-heavy paths, and CI workflows.

### Devnet Sources And Constraints

Official Solana docs used for planning:

- Clusters and public endpoints: https://solana.com/docs/references/clusters
- RPC overview: https://solana.com/docs/rpc
- `requestAirdrop`: https://solana.com/docs/rpc/http/requestairdrop
- `sendTransaction`: https://solana.com/docs/rpc/http/sendtransaction
- `getSignatureStatuses`: https://solana.com/docs/rpc/http/getsignaturestatuses
- `getTransaction`: https://solana.com/docs/rpc/http/gettransaction
- `getBalance`: https://solana.com/docs/rpc/http/getbalance
- Transactions: https://solana.com/docs/core/transactions

Planning implications:

- Devnet is the correct public cluster for developer/evaluator experimentation.
- Devnet tokens are not real and devnet may reset; the report must state this.
- Public RPC endpoints are shared, rate-limited infrastructure and are not production-grade.
- `sendTransaction` acceptance does not prove confirmation; the demo must check status/transaction evidence after submission.
- Balance movement evidence should use before/after `getBalance` at the configured commitment.
- Faucet/rate-limit failures are expected external blockers; they must produce `NO-GO: devnet_unavailable` or equivalent, not a local-only pass.

## MVP Claim Matrix

The final proof report must include a machine-readable claim matrix and a human-readable rendering.

Required claim classes:

| Claim class | Meaning | MVP rule |
| --- | --- | --- |
| `real` | Executed against actual KAMN local runtime code/processes in the demo run | Required for runtime, service API, auth, message/task, relay, websocket, audit |
| `devnet-backed` | Executed against Solana devnet with transaction/status/balance evidence | Required for settlement or asset movement |
| `local-only` | Executed locally with no external value movement | Allowed for KAMN runtime, auth, message/task, state, relay, websocket, audit |
| `dry-run` | Planned or simulated command only | Not allowed for MVP completion claims |
| `placeholder` | Deterministic fixture or placeholder detail | Not allowed in required MVP proof path |
| `roadmap` | Explicitly not implemented/proven in MVP | Required for production readiness, mainnet, generalized settlement, consensus/finality |

Required final report checks:

- No required MVP claim can be `dry-run`.
- No required MVP claim can contain a placeholder detail.
- Any claim containing `escrow`, `settlement`, `exchange`, `asset`, `transfer`, `lamports`, or `value movement` must be `devnet-backed`.
- `devnet-backed` settlement evidence must include:
  - `network=solana:devnet`
  - `rpc_url`
  - payer pubkey
  - recipient pubkey
  - lamports requested
  - settlement transaction signature
  - configured commitment
  - transaction status proof
  - transaction detail or slot proof
  - recipient balance before/after and observed delta
  - persisted KAMN linkage (`settlement_tx_signature`, `settlement_network`, `settlement_commitment`)
  - repeated-release idempotency evidence that the same escrow does not submit a second transfer

## Target Product Story

The evaluator story should read like this:

1. "KAMN starts a local service-api runtime with durable state files under one demo run directory."
2. "Alice and Bob have authenticated agent identities and signed requests."
3. "Alice sends Bob a task/message through KAMN."
4. "The message is persisted, placed in durable relay/projection state, visible to Bob, and observable on websocket events."
5. "Bob accepts/completes the task or produces an artifact."
6. "KAMN exports audit/proof evidence for the message/task lifecycle."
7. "KAMN releases a bounded escrow through the existing service-api settlement path."
8. "The settlement claim is backed by a real Solana devnet transaction, status check, transaction/balance evidence, persisted KAMN signature linkage, and idempotent repeated-release behavior."
9. "The final report states exactly what was local, what was devnet-backed, what was dry-run, what was not claimed, and what remains roadmap."

Do not use `InMemoryKamnClient` for this story except as a negative control or legacy comparison. The demo must use real local KAMN runtime processes and service API routes.

## GitHub Issue And Spec Breakdown

Open one parent tracking issue, then the child issues below. The parent issue is not an implementation issue; it tracks the MVP umbrella and links child specs/PRs.

Parent issue:

- Title: `feat: KAMN MVP demo readiness tracker`
- Body must include problem statement, acceptance criteria, non-goals, and links to this plan plus the brainstorm.
- Non-goal: do not implement code directly under the parent issue.

First 5 child issues, in dependency order:

### Issue 1: Restore Local Quality Gates Before MVP Expansion

Suggested title: `fix: restore local formatting and strict clippy gates`

Purpose:

- Make `make check` green before expanding MVP features.

Spec path after issue creation:

- `specs/<issue>-restore-local-quality-gates.md`

Acceptance criteria:

- `cargo fmt --check` passes.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- `make check` passes.
- No clippy/test weakening is introduced.
- If scope is too large, split by crate/module but do not start feature work until all split gate-recovery issues are green.

Red evidence:

- Existing `cargo fmt --check` failure.
- Existing strict workspace clippy failure in `kamn-core`.

Likely touched surfaces:

- Rust files across `crates/kamn-core`, `crates/kamn-node`, `crates/kamn-sdk`, and support tests.
- Avoid touching scripts/workflows unless the gate itself is incorrectly configured.

Completion evidence:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
make check
cargo test -p kamn-core
cargo test -p kamn-node
```

Risk gate:

- Do not add `#[allow(...)]`, relax lint levels, remove tests, or demote warnings to make the gate green unless the issue/spec explicitly justifies a narrow false positive.

### Issue 2: Define MVP Claim Matrix And Proof Contract

Suggested title: `feat: define MVP demo proof contract and claim taxonomy`

Purpose:

- Freeze the proof/report schema before implementing the runner.
- Make overclaiming impossible in the generated report.

Spec path after issue creation:

- `specs/<issue>-mvp-demo-proof-contract.md`

Acceptance criteria:

- A checked-in schema or Rust model defines the claim matrix and report fields.
- A contract test fails if required MVP claims are missing.
- A contract test fails if a required claim is `dry-run` or `placeholder`.
- A contract test fails if settlement/asset wording is not `devnet-backed`.
- A docs/runbook section explains the allowed claim classes.

Red tests first:

```bash
cargo test -p kamn-e2e-harness --test mvp_demo_claim_contract -- --nocapture
cargo test -p kamn-core --test mvp_demo_claim_docs_contract -- --nocapture
```

Expected initial failure:

- No MVP report schema exists.
- No required claim matrix exists.
- No placeholder/dry-run rejection contract exists.

Likely touched surfaces:

- `crates/kamn-e2e-harness/src/verify/`
- `crates/kamn-e2e-harness/tests/`
- `docs/validation/`
- `docs/developer/readme-contract-reference.md` only if needed for discoverability.

Completion evidence:

```bash
cargo test -p kamn-e2e-harness --test mvp_demo_claim_contract -- --nocapture
cargo test -p kamn-core --test mvp_demo_claim_docs_contract -- --nocapture
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Risk gate:

- Do not write a report schema that permits ambiguous labels such as `live-ish`, `simulated-live`, or `real-ish`.

### Issue 3: Add Canonical Local KAMN MVP Demo Command

Suggested title: `feat: add canonical local KAMN MVP demo command`

Purpose:

- Provide the local, evaluator-friendly product story without relying on in-memory SDK demo paths.

Spec path after issue creation:

- `specs/<issue>-canonical-local-mvp-demo-command.md`

Acceptance criteria:

- `make demo-mvp` exists.
- The command starts real local KAMN service-api runtime processes or a real in-process service API server backed by the same runtime code.
- The command writes all state/evidence under a run directory, for example `.kamn/demo/<run-id>/`.
- The command proves authenticated agent identity, signed request or equivalent service-auth evidence, message/task flow, durable state, relay/projection, websocket event visibility, audit/proof export, and human-readable report generation.
- The command labels this phase's runtime story as `real` and `local-only`, except settlement, which must be absent or marked `not-claimed` until Issue 4.
- The command must fail if required evidence files are missing.
- The command must not count harness `deterministic placeholder` infra/deploy details as proof.

Red tests first:

```bash
cargo test -p kamn-e2e-harness --test mvp_demo_local_runtime_contract -- --nocapture
cargo test -p kamn-node --test mvp_demo_service_api_contract -- --nocapture
bash scripts/ci/test_readme_contract.sh
```

Expected initial failure:

- `make demo-mvp` does not exist.
- No single report ties local runtime, auth, relay, websocket, audit, and proof together.

Likely touched surfaces:

- `Makefile`
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/src/main.rs` if the binary dispatch is split there.
- `crates/kamn-e2e-harness/src/verify/`
- `crates/kamn-e2e-harness/tests/`
- `crates/kamn-sdk/src/live.rs`
- `crates/kamn-sdk/src/service_client*.rs`
- `crates/kamn-node/src/service_api_endpoint/*` only if a real route/report gap is found.
- `docs/validation/mvp-demo-slice.md`
- `docs/validation/current-proven-runtime-slices.md`

Shell-surface process note:

- Because `Makefile` is touched, the GitHub issue body must include the shell-surface DoR estimates from `AGENTS.md`.
- The PR summary/closure must include shell-surface DoD actuals.

Completion evidence:

```bash
make demo-mvp
test -f .kamn/demo/latest/proof/report.json
test -f .kamn/demo/latest/proof/report.md
cargo test -p kamn-e2e-harness --test mvp_demo_local_runtime_contract -- --nocapture
cargo test -p kamn-node integration_service_api_endpoint_working_vertical_slice_proves_delivery_dispatch_and_audit_evidence -- --exact --nocapture
cargo test -p kamn-node integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract -- --exact --nocapture
cargo test -p kamn-node regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart -- --exact --nocapture
```

Risk gate:

- If the runner uses `kamn-e2e-harness run`, first remove or quarantine placeholder infra/deploy records from required MVP proof. Placeholder records may remain as roadmap/development metadata but cannot satisfy required claims.

### Issue 4: Make Settlement And Asset Movement Devnet-Backed In The MVP Demo

Suggested title: `feat: require Solana devnet-backed settlement evidence in MVP demo`

Purpose:

- Connect the MVP demo report to the existing Solana devnet asset-movement lane whenever settlement or asset movement is claimed.

Spec path after issue creation:

- `specs/<issue>-mvp-demo-devnet-backed-settlement.md`

Acceptance criteria:

- The MVP demo can run with `KAMN_MVP_DEVNET_MODE=required`.
- The demo prepares or consumes devnet keypair material without committing secrets.
- The demo funds or verifies the devnet payer account.
- The demo executes escrow release through the real service-api settlement path.
- The demo records a real Solana devnet transaction signature and commitment.
- The demo verifies transaction status and transaction/balance evidence after submission.
- The KAMN persisted escrow record contains `settlement_tx_signature`, `settlement_network=solana:devnet`, and `settlement_commitment`.
- Repeated release for the same escrow reuses the same persisted transaction linkage.
- If devnet RPC, faucet, transaction, confirmation, or balance proof fails, the report is `NO-GO` and does not claim settlement success.

Red tests first:

```bash
cargo test -p kamn-e2e-harness --test mvp_demo_devnet_settlement_contract -- --nocapture
cargo test -p kamn-node --test solana_devnet_asset_movement_slice_contract -- --nocapture
cargo test -p kamn-node --bin kamn-node 'main_tests::service_api_endpoint_tests::task_escrow_persistence_contract_tests::task_escrow_solana_asset_movement_contract_tests::integration_service_api_endpoint_live_solana_asset_movement_release_persists_transaction_signature_metadata' -- --exact --nocapture
```

Expected initial failure:

- MVP report does not yet include devnet-backed settlement fields.
- `make demo-mvp` does not yet gate settlement wording on devnet evidence.

Live proof command:

```bash
KAMN_MVP_DEVNET_MODE=required \
KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com \
make demo-mvp
```

Operator confirmation command using existing ignored proof:

```bash
cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::task_escrow_persistence_contract_tests::task_escrow_solana_asset_movement_live_contract_tests::integration_service_api_endpoint_live_solana_asset_movement_release_submits_real_devnet_transfer' \
  -- --ignored --exact --nocapture
```

Likely touched surfaces:

- `crates/kamn-e2e-harness/src/`
- `crates/kamn-e2e-harness/tests/`
- `crates/kamn-node/src/service_api_endpoint/live_settlement_dispatch/*`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/task_escrow_ops.rs`
- `crates/kamn-node/tests/solana_devnet_asset_movement_slice_contract.rs`
- `docs/validation/solana-devnet-asset-movement-slice.md`
- `docs/validation/mvp-demo-slice.md`

Completion evidence:

```bash
cargo test -p kamn-node --test solana_devnet_asset_movement_slice_contract -- --nocapture
cargo test -p kamn-e2e-harness --test mvp_demo_devnet_settlement_contract -- --nocapture
KAMN_MVP_DEVNET_MODE=required KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com make demo-mvp
```

Risk gates:

- No fallback from devnet settlement to local-only release when `KAMN_MVP_DEVNET_MODE=required`.
- No "asset movement" wording unless balance delta and transaction evidence are present.
- No mainnet support in the MVP.
- Public devnet faucet/rate-limit failure is `NO-GO: devnet_unavailable`, not a test pass.

### Issue 5: Wire Evaluator Report, Verification, Docs, And CI Lanes

Suggested title: `integrate: wire MVP demo proof report and evaluator verification`

Purpose:

- Make the demo easy to run, easy to verify, and honest in CI/manual workflows.

Spec path after issue creation:

- `specs/<issue>-mvp-demo-report-verification-ci.md`

Acceptance criteria:

- The demo emits both:
  - `.kamn/demo/<run-id>/proof/report.json`
  - `.kamn/demo/<run-id>/proof/report.md`
- A verifier command reads `report.json` and fails closed on missing/ambiguous/overstated claims.
- `report.md` is evaluator-readable without requiring source inspection.
- README or validation docs advertise one command sequence and the non-claims.
- PR CI runs deterministic contract tests and report-schema tests.
- Scheduled/manual CI can run devnet-backed proof when credentials/RPC/faucet availability are present.
- CI and docs separate PR confidence from scheduled/live confidence.

Red tests first:

```bash
cargo test -p kamn-e2e-harness --test mvp_demo_report_verifier_contract -- --nocapture
cargo test -p kamn-core --test ci_strategy_docs doc_contains_mvp_demo_lane_boundaries -- --exact
bash scripts/ci/test_ci_tools.sh
```

Expected initial failure:

- No verifier command exists.
- CI strategy lacks MVP demo lane boundary markers.
- README/validation docs do not advertise the final command/report contract.

Likely touched surfaces:

- `README.md`
- `docs/validation/mvp-demo-slice.md`
- `docs/validation/current-proven-runtime-slices.md`
- `docs/ci/strategy.md`
- `.github/workflows/e2e-live.yml` only if adding scheduled/manual MVP run is necessary.
- `scripts/ci/test_ci_tools.sh` only for deterministic contract coverage.
- `crates/kamn-e2e-harness/src/verify/`

Shell/workflow surface process note:

- If scripts, workflows, issue templates, or `Makefile` are touched, follow `AGENTS.md` shell-surface DoR/DoD fields exactly.

Completion evidence:

```bash
make demo-mvp
cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json
cargo test -p kamn-e2e-harness --test mvp_demo_report_verifier_contract -- --nocapture
cargo test -p kamn-core --test ci_strategy_docs doc_contains_mvp_demo_lane_boundaries -- --exact
bash scripts/ci/test_ci_tools.sh
```

Risk gate:

- Do not require live devnet proof in the hot PR fast gate unless the repo accepts rate-limit/flakiness risk. Prefer PR schema/local deterministic proof plus manual/scheduled live devnet proof.

## Phased Execution Plan

### Phase 0: Preflight, Issue Creation, And Branch Discipline

What this proves:

- Work starts from the latest appropriate branch and obeys repo process.

Actions:

1. Run branch refresh:

   ```bash
   git fetch --all --prune
   git status --short --branch
   gh pr list --repo njfio/kamn --state open --limit 50
   ```

2. Create parent tracker issue.
3. Create Issue 1 with acceptance criteria and non-goals.
4. Create `specs/<issue>-restore-local-quality-gates.md`.
5. Commit spec before any tests or implementation.

Likely touched files:

- `specs/<issue>-restore-local-quality-gates.md`

Risk gate:

- Do not implement anything before the issue exists and the spec is committed.

Rollback/checkpoint:

- If issue creation fails due missing GitHub auth, stop execution and report the blocker. Do not start code.
- If branch state changes underfoot, fetch again and re-check whether a newer non-dependency product branch supersedes `main`.

### Phase 1: Restore Local Gate Health

What this proves:

- The repo can be iterated honestly under its own `make check` contract.

Actions:

1. Preserve red evidence from:

   ```bash
   cargo fmt --check
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   ```

2. Apply formatting with `cargo fmt`.
3. Fix strict clippy by category:
   - unused imports/mut
   - mechanical lint suggestions
   - test `unwrap`/`unwrap_err` replacements
   - module inception and type-complexity extractions
   - missing docs on exported items
4. Run focused tests for touched modules.
5. Run `make check`.
6. Commit in repo-required TDD arc:
   - spec commit
   - red evidence/test commit if new regression tests are needed
   - green implementation commit
   - mandatory refactor commit
   - integration/gate evidence commit if required

Likely touched files:

- Many Rust files under `crates/kamn-core/src/`
- Some Rust tests under `crates/kamn-core/tests/`
- Possibly `crates/kamn-sdk/tests/`

Red tests first:

- Existing red `cargo fmt --check`
- Existing red strict clippy

Acceptance criteria:

- `make check` is green.
- No lint/test weakening.
- Touched functions/files respect AGENTS size rules or have follow-up issues where pre-existing debt is out of scope.

Verification commands:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
make check
cargo test -p kamn-core
cargo test -p kamn-node
```

Risk gates:

- If missing docs are numerous, prefer documenting stable public API items over adding broad allow attributes.
- If this becomes too large, split Issue 1 into crate/module sub-issues, but keep all MVP feature issues blocked until gate recovery is complete.

Rollback/checkpoint:

- Commit after mechanical formatting separately from semantic lint fixes.
- If semantic lint fixes introduce failures, revert only the current lint-fix commit and keep the formatting commit if it is clean.

### Phase 2: Freeze MVP Claim Contract

What this proves:

- The final demo cannot overclaim.

Actions:

1. Create Issue 2 and spec.
2. Add red report-schema/claim-matrix tests.
3. Define required claims and statuses.
4. Add negative fixtures:
   - settlement claim marked `local-only`
   - required runtime claim marked `dry-run`
   - placeholder text in required proof
   - missing devnet transaction signature
   - balance delta missing for asset movement
5. Implement verifier/model.
6. Wire docs contract.

Likely touched files:

- `crates/kamn-e2e-harness/src/verify/`
- `crates/kamn-e2e-harness/tests/`
- `fixtures/` if the repo already has a suitable fixture pattern
- `docs/validation/mvp-demo-slice.md`

Red tests first:

```bash
cargo test -p kamn-e2e-harness --test mvp_demo_claim_contract -- --nocapture
```

Completion evidence:

- Positive report fixture passes.
- Negative report fixtures fail with deterministic reason codes:
  - `mvp_claim_required_missing`
  - `mvp_claim_placeholder_forbidden`
  - `mvp_claim_dry_run_forbidden`
  - `mvp_settlement_requires_devnet_backing`
  - `mvp_devnet_signature_missing`
  - `mvp_devnet_balance_delta_missing`

Verification commands:

```bash
cargo test -p kamn-e2e-harness --test mvp_demo_claim_contract -- --nocapture
cargo test -p kamn-core --test mvp_demo_claim_docs_contract -- --nocapture
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Risk gates:

- The verifier must be fail-closed and deterministic.
- Human-readable report rendering must derive from the same JSON, not a separate hand-written summary.

Rollback/checkpoint:

- If report schema design churns, keep negative tests and adjust only the model/rendering. Do not weaken the negative tests.

### Phase 3: Build The Local Runtime Demo Spine

What this proves:

- A local evaluator can run a real KAMN runtime story from one command without reading scattered test docs.

Actions:

1. Create Issue 3 and spec.
2. Add red test asserting `make demo-mvp` contract or Rust runner command exists.
3. Implement a Rust-owned runner, preferably in `kamn-e2e-harness`, that:
   - creates `.kamn/demo/<run-id>/`
   - starts local `kamn-node` service API runtime(s) on loopback
   - sets explicit state, relay spool, audit export, TLS-disabled-loopback-only, and auth env vars
   - waits for `/healthz`
   - opens websocket event listener
   - sends signed/authenticated service API requests
   - creates message/task flow
   - verifies recipient-visible message/task status
   - restarts or reopens state enough to prove durable state and relay/projection
   - captures audit export
   - writes report JSON/Markdown
4. Add `make demo-mvp` as the thin wrapper.
5. Update validation docs.

Likely touched files:

- `Makefile`
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/src/verify/`
- `crates/kamn-e2e-harness/tests/`
- `crates/kamn-sdk/src/service_client*.rs`
- `crates/kamn-node/src/service_api_endpoint/*` only for missing route/report evidence
- `docs/validation/mvp-demo-slice.md`

Red tests first:

```bash
cargo test -p kamn-e2e-harness --test mvp_demo_local_runtime_contract -- --nocapture
```

Completion evidence required in report:

- git SHA and branch
- exact command line
- generated run id
- local node bind address(es)
- health check status
- state file path and hash
- relay spool path and before/after status
- Alice/Bob DIDs or agent identifiers
- auth/signature/replay evidence
- message id and status progression
- task id and state progression
- websocket event frames with increasing sequence
- audit export file path and hash
- claim matrix with runtime claims as `real`/`local-only`
- settlement claim absent or `not-claimed` until Issue 4

Verification commands:

```bash
make demo-mvp
cargo test -p kamn-e2e-harness --test mvp_demo_local_runtime_contract -- --nocapture
cargo test -p kamn-node integration_service_api_endpoint_working_vertical_slice_proves_delivery_dispatch_and_audit_evidence -- --exact --nocapture
cargo test -p kamn-node integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract -- --exact --nocapture
cargo test -p kamn-node regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart -- --exact --nocapture
```

Risk gates:

- Do not use `InMemoryKamnClient` for MVP proof.
- Do not accept a report if required evidence is sourced from deterministic placeholder text.
- Do not require a separate Kolme checkout for the minimal local KAMN MVP unless the issue/spec proves it is necessary.

Rollback/checkpoint:

- Keep the runner behind a new command until stable.
- If process orchestration is flaky, first reduce the local story to one KAMN node plus service API state proof, then add multi-process relay once deterministic.

### Phase 4: Add Devnet-Backed Settlement And Asset Movement

What this proves:

- Any settlement or asset-movement claim in the MVP is backed by real Solana devnet execution/evidence.

Actions:

1. Create Issue 4 and spec.
2. Add red tests that reject settlement wording without devnet evidence.
3. Reuse the existing `live_settlement_dispatch` path.
4. Add runner support for:
   - auto-generated ephemeral devnet keypair JSON, or explicit keypair file
   - recipient pubkey generation or explicit recipient
   - devnet payer funding/preflight
   - `KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL`
   - `KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE`
   - `KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY`
   - `KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS`
   - `KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT=finalized`
5. Call the real service API escrow release path.
6. Query devnet evidence after release.
7. Verify repeated release idempotency.
8. Write devnet-backed claim fields into the report.

Likely touched files:

- `crates/kamn-e2e-harness/src/`
- `crates/kamn-e2e-harness/tests/`
- `crates/kamn-node/src/service_api_endpoint/live_settlement_dispatch/*`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/task_escrow_ops.rs`
- `docs/validation/mvp-demo-slice.md`
- `docs/validation/solana-devnet-asset-movement-slice.md`

Red tests first:

```bash
cargo test -p kamn-e2e-harness --test mvp_demo_devnet_settlement_contract -- --nocapture
cargo test -p kamn-node --test solana_devnet_asset_movement_slice_contract -- --nocapture
```

Completion evidence required in report:

- `claim_class=devnet-backed`
- `settlement_network=solana:devnet`
- devnet RPC URL
- payer pubkey
- recipient pubkey
- lamports
- payer balance before/after if collected
- recipient balance before/after
- settlement transaction signature
- transaction status at configured commitment
- transaction detail or slot proof
- persisted KAMN escrow state with the same transaction signature
- repeated release returns same signature and does not emit second transfer
- clear statement: devnet tokens are not real, devnet can reset, this is not mainnet/production settlement

Verification commands:

```bash
KAMN_MVP_DEVNET_MODE=required \
KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com \
make demo-mvp

cargo test -p kamn-node --test solana_devnet_asset_movement_slice_contract -- --nocapture

cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::task_escrow_persistence_contract_tests::task_escrow_solana_asset_movement_live_contract_tests::integration_service_api_endpoint_live_solana_asset_movement_release_submits_real_devnet_transfer' \
  -- --ignored --exact --nocapture
```

Risk gates:

- If devnet faucet is rate-limited, support an explicit pre-funded devnet keypair path. Do not fake funding.
- If `sendTransaction` succeeds but confirmation/status lookup fails, report `NO-GO`.
- If balance delta does not match expected lamports, report `NO-GO`.
- If the KAMN state linkage does not match the devnet signature, report `NO-GO`.

Rollback/checkpoint:

- Keep local runtime demo from Phase 3 working even when devnet is unavailable, but label it `local-only` and `settlement_not_claimed`.
- Do not merge Issue 4 as MVP-complete without at least one successful devnet-backed evidence run captured in the PR or release evidence.

### Phase 5: Evaluator Report, Verifier, Docs, And CI

What this proves:

- The result is understandable and independently checkable.

Actions:

1. Create Issue 5 and spec.
2. Add report verifier command.
3. Add human-readable report template derived from JSON.
4. Add docs:
   - `docs/validation/mvp-demo-slice.md`
   - README quick path update
   - CI strategy lane boundaries
5. Add deterministic CI contract tests.
6. Add manual/scheduled live devnet lane only if needed.

Likely touched files:

- `README.md`
- `docs/validation/mvp-demo-slice.md`
- `docs/validation/current-proven-runtime-slices.md`
- `docs/ci/strategy.md`
- `crates/kamn-e2e-harness/src/verify/`
- `crates/kamn-e2e-harness/tests/`
- `.github/workflows/e2e-live.yml` only if live lane wiring is required
- `scripts/ci/test_ci_tools.sh` only if deterministic CI regression command is needed

Red tests first:

```bash
cargo test -p kamn-e2e-harness --test mvp_demo_report_verifier_contract -- --nocapture
cargo test -p kamn-core --test ci_strategy_docs doc_contains_mvp_demo_lane_boundaries -- --exact
```

Completion evidence:

```bash
make demo-mvp
cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json
cargo test -p kamn-e2e-harness --test mvp_demo_report_verifier_contract -- --nocapture
bash scripts/ci/test_ci_tools.sh
```

Risk gates:

- PR fast gate should not depend on live public RPC unless the repo explicitly accepts the flake/cost tradeoff.
- Scheduled/manual live proof must upload report artifacts and preserve `NO-GO` when external devnet is unavailable.

Rollback/checkpoint:

- If CI live lane is flaky, keep deterministic verifier tests in PR CI and make live devnet proof manual/scheduled with artifact upload.
- Do not remove the local evaluator command while iterating on CI.

### Phase 6: Demo Rehearsal And Release Readiness

What this proves:

- The MVP path is usable by someone other than the implementer.

Actions:

1. Fresh clone rehearsal on a clean machine or clean worktree.
2. Run:

   ```bash
   make check
   KAMN_MVP_DEVNET_MODE=required KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com make demo-mvp
   cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json
   ```

3. Read `report.md` without source context.
4. Confirm non-claims are visible:
   - not production-ready
   - not mainnet
   - not generalized settlement
   - not consensus/finality beyond bounded evidence
   - not global fault tolerance
5. Record known residual risks.

Completion evidence:

- One final proof bundle from fresh checkout.
- One verifier pass.
- One PR description with issue link, spec link, command evidence, shell-surface actuals if applicable, and explicit non-claims.

Risk gates:

- If a fresh checkout cannot run the command from docs, MVP is not complete.
- If the report requires maintainers to inspect source code to understand proof status, MVP is not complete.

Rollback/checkpoint:

- If the rehearsal fails due local runner behavior, fix before PR.
- If the rehearsal fails only due public devnet/faucet outage, rerun with pre-funded devnet keypair or private devnet RPC and label the evidence source.

## Verification Command Matrix

Always run after Phase 1:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
make check
```

Core local runtime proof commands:

```bash
cargo test -p kamn-node integration_service_api_endpoint_working_vertical_slice_proves_delivery_dispatch_and_audit_evidence -- --exact --nocapture
cargo test -p kamn-node integration_runtime_daemon_without_route_map_preserves_relay_spool_entries -- --exact --nocapture
cargo test -p kamn-node integration_service_api_endpoint_cross_node_relay_delivery_contract -- --exact --nocapture
cargo test -p kamn-node integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract -- --exact --nocapture
cargo test -p kamn-node regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart -- --exact --nocapture
cargo test -p kamn-node integration_service_api_endpoint_persists_task_and_escrow_state_across_restart -- --exact --nocapture
cargo test -p kamn-node --test live_solana_bridge_websocket_slice_contract -- --nocapture
```

Solana/devnet proof commands:

```bash
python3 scripts/runtime/run_live_solana_devnet_proof.py \
  --rpc-url https://api.devnet.solana.com \
  --output-json /tmp/live-solana-devnet-report.json

python3 scripts/runtime/validate_live_solana_devnet_proof.py \
  --report-file /tmp/live-solana-devnet-report.json \
  --output-json /tmp/live-solana-devnet-validation.json

KAMN_SOLANA_DEVNET_REPORT_FILE=/tmp/live-solana-devnet-report.json \
KAMN_SOLANA_DEVNET_NORMALIZATION_REPORT=/tmp/live-solana-devnet-normalization.json \
  cargo test -p kamn-core --test live_solana_devnet_receipt_normalization -- --nocapture

python3 scripts/runtime/check_live_solana_devnet_proof_policy.py \
  --validation-report-file /tmp/live-solana-devnet-validation.json \
  --normalization-report-file /tmp/live-solana-devnet-normalization.json \
  --expected-final-decision GO \
  --output-json /tmp/live-solana-devnet-policy.json
```

Existing asset-movement commands:

```bash
cargo test -p kamn-node --test solana_devnet_asset_movement_slice_contract -- --nocapture

cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::task_escrow_persistence_contract_tests::task_escrow_solana_asset_movement_contract_tests::integration_service_api_endpoint_live_solana_asset_movement_release_persists_transaction_signature_metadata' \
  -- --exact --nocapture

cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::task_escrow_persistence_contract_tests::task_escrow_solana_asset_movement_contract_tests::integration_service_api_endpoint_live_solana_asset_movement_release_is_idempotent_for_repeated_submit' \
  -- --exact --nocapture

cargo test -p kamn-node --bin kamn-node \
  'main_tests::service_api_endpoint_tests::task_escrow_persistence_contract_tests::task_escrow_solana_asset_movement_contract_tests::integration_service_api_endpoint_live_solana_asset_movement_release_reuses_signature_after_restart' \
  -- --exact --nocapture
```

Final MVP commands:

```bash
make demo-mvp
KAMN_MVP_DEVNET_MODE=required KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com make demo-mvp
cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json
```

## Rollback And Checkpoint Strategy

Branching:

- Use repo-required issue branches: `<issue>-<slug>`.
- Do not push directly to `main`.
- Keep the parent tracker separate from child implementation branches.

Commits:

- Commit the spec before tests or implementation.
- Preserve red -> green -> refactor -> integrate sequence.
- Use Lore protocol trailers in commits.
- Keep formatting-only commits separate from semantic commits.

Checkpoints:

- Checkpoint after each issue's spec commit.
- Checkpoint after red tests demonstrate the gap.
- Checkpoint after green implementation.
- Checkpoint after mandatory refactor.
- Checkpoint after integration/proof evidence.

Rollback:

- If gate recovery breaks tests, revert only the current semantic lint-fix commit.
- If the demo runner becomes flaky, keep the last passing local-only runner and isolate devnet work behind `KAMN_MVP_DEVNET_MODE=required`.
- If public devnet is unavailable, keep the report `NO-GO` and rerun with a pre-funded devnet keypair or private devnet RPC. Do not merge a fake settlement pass.
- If CI live proof is flaky, move live proof to manual/scheduled and keep deterministic verifier/schema tests in PR CI.

Evidence retention:

- Demo artifacts should live under `.kamn/demo/<run-id>/`.
- `.kamn/demo/latest` may be a local symlink or pointer file, but build artifacts and secrets must not be committed.
- PR evidence should include report excerpts and artifact paths, not committed private key material.

## Do Not Build Yet

Do not spend MVP time on:

- Mainnet settlement or production custody.
- Generalized multi-chain settlement.
- Full bridge finality.
- Byzantine-safe dispute resolution.
- Broad multi-node consensus/finality.
- Global fault-tolerance under arbitrary partitions.
- UI dashboard or marketing landing page.
- New SDK language surfaces.
- More governance/spec/doc taxonomies not needed by the demo.
- New dependencies unless an issue/spec explicitly justifies them and the user approves.
- Refactoring unrelated modules beyond what is needed for gate recovery and touched-surface size limits.
- Replacing the whole runtime architecture.
- Treating `InMemoryKamnClient` as proof for exchange, escrow, settlement, or asset movement.

## Recommended Execution Mode

Recommended next workflow: `ce:work` issue-by-issue, starting with Issue 1.

Reason:

- The immediate blocker is sequential and gate-sensitive: `make check` must become green before MVP expansion.
- The repo process requires issue-first, spec-before-code, TDD, mandatory refactor, and integration wiring. `ce:work` is the right long-running implementation mode for that.
- Do not start with `team`: parallel implementation before gate recovery will create conflicts across many Rust files.
- After Issue 2 freezes the claim contract, consider a small `team` split:
  - one lane for local demo runner,
  - one lane for report verifier/docs,
  - one lane for devnet proof hardening.
- If running under OMX runtime and persistent autonomous completion is desired, use a `ralph`-style single-owner loop after the first two issues are fully specified and the gate recovery plan is accepted.

Default next action:

1. Open parent tracker issue.
2. Open Issue 1.
3. Write and commit `specs/<issue>-restore-local-quality-gates.md`.
4. Start `ce:work` on Issue 1 only.

Do not start Issues 2-5 until `make check` is green or Issue 1 has been deliberately split into smaller gate-recovery sub-issues that together restore `make check`.
