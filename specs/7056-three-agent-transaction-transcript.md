# 7056 Three-Agent Transaction Transcript

## Objective

Add a durable MVP proof artifact that records the three-agent transaction story
behind `three_agent_escrow_verification`. The report should no longer rely only
on inline run-id-derived fields; it should point to an inspectable transcript
showing Agent A, Agent B, and Agent C as distinct perspectives over the same
task/transaction, escrow, and devnet-backed settlement evidence.

## Inputs/Outputs

Inputs:
- MVP `run_id`.
- Successful `DevnetSettlementEvidence` from the devnet-required demo path.
- Existing local proof artifacts under `.kamn/demo/<run-id>/proof/`.

Outputs:
- `.kamn/demo/<run-id>/proof/three-agent-transcript.json`.
- A `three_agent_transcript` artifact entry in successful devnet-backed reports.
- `three_agent_escrow_verification` fields for transcript artifact path and
  deterministic transcript digest/commitment.
- Human-readable report text that points to the transcript without exposing raw
  private payloads.

## Boundaries/Non-goals

- The transcript is local proof evidence. It does not create a new production
  service architecture.
- Settlement, escrow, asset movement, lamports, or value movement claims remain
  devnet-backed and must be linked to the Solana devnet settlement evidence.
- No raw participant-private payloads in report JSON, report Markdown, or the
  transcript artifact.
- No mainnet, production-readiness, generalized exchange, dispute, bridge, or
  broad privacy guarantee.
- No new dependency unless this spec is amended first.

## Failure Modes

- Devnet-backed report omits the transcript artifact path.
- `three_agent_escrow_verification` omits transcript path or digest fields.
- Transcript artifact is missing, malformed, or not labelled as local proof
  evidence linked to devnet-backed settlement.
- Transcript omits Agent A registration, Agent B registration, task invocation,
  task acceptance, escrow funding/release, or Agent C verification steps.
- Transcript contains raw participant-private payload markers.
- Transcript settlement signature, amount, payer, recipient, or commitment
  conflicts with the report's devnet-backed settlement evidence.
- Local-only reports incorrectly claim transcript or settlement success.

## Acceptance Criteria

- [x] Successful devnet-backed reports include an artifact entry named
  `three_agent_transcript`.
- [x] Successful `three_agent_escrow_verification` claims include
  `three_agent_transcript_artifact` and `three_agent_transcript_digest`.
- [x] `three-agent-transcript.json` records Agent A registration, Agent B
  registration, Agent A invocation, Agent B acceptance, escrow fund, escrow
  release, and Agent C verification steps.
- [x] The transcript records Agent A and Agent B participant-private views and
  Agent C restricted-public verifier view.
- [x] The transcript records the devnet settlement signature, amount, payer,
  recipient, and finalized commitment from the report's devnet-backed evidence.
- [x] `verify-mvp-demo` rejects devnet-backed reports missing transcript fields.
- [x] `verify-mvp-demo` rejects report-file verification when the transcript
  artifact is missing, malformed, raw-private-leaking, or mismatched with the
  report claim.
- [x] Local-only reports do not include the transcript artifact entry and remain
  valid without a three-agent transcript.
- [x] Report Markdown surfaces the transcript artifact and reiterates that raw
  private payloads are redacted.

## Files to Touch

- `crates/kamn-e2e-harness/src/mvp_demo/report.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/report_artifacts.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/report_markdown.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/runner.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_claim.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_transcript.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_verify.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract/support.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract/three_agent.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_claim_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_three_agent_claim_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_three_agent_transcript_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_evaluator_demo_runbook_contract.rs`
- `docs/validation/mvp-evaluator-demo.md`

## Error Semantics

- Missing or malformed transcript fields fail closed with explicit
  `Err(String)` verifier messages.
- Command-level report verification must fail if the report points to a missing
  or mismatched transcript artifact.
- No fallback may convert local-only or missing devnet settlement evidence into
  a transcript-backed settlement claim.
- Transcript writing errors must fail `make demo-mvp`; do not silently omit the
  artifact from a devnet-backed report.

## Test Plan

Red:
- Add verifier tests that reject devnet-backed three-agent claims missing
  transcript artifact/digest fields.
- Add command-level verifier tests that reject missing, raw-private-leaking, and
  mismatched transcript artifacts.
- Add a local-only report assertion that no transcript artifact is claimed.

Green:
- Generate `three-agent-transcript.json` after successful devnet settlement
  evidence.
- Add transcript artifact and digest fields to the devnet-backed report.
- Add command-level artifact validation against report claim fields and devnet
  settlement fields.
- Update Markdown and runbook wording.

Refactor:
- Keep new transcript generation and validation helpers single-purpose.
- Split files before they exceed the repo line-budget guidance.
- Reuse existing flat JSON marker/extractor helpers; do not add dependencies.

Integration:
- Generate local-only and devnet-required reports.
- Run canonical verifier against both.
- Run Pi evidence extraction against the devnet report to ensure Pi still sees
  the updated three-agent boundary.

## Completion Evidence

- `cargo test -p kamn-e2e-harness --test mvp_demo_three_agent_claim_contract -- --nocapture`
- `cargo test -p kamn-e2e-harness --test mvp_demo_agent_harness_claim_contract -- --nocapture`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `make check`
- `make demo-mvp`
- Devnet-required `make demo-mvp` or explicit Solana devnet NO-GO evidence.
- `cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json`
- Pi extension smoke or extraction against the generated report.

Completed:
- Red evidence captured:
  `mvp_demo_three_agent_transcript_contract` failed 4 expected cases because
  current verification accepted missing transcript fields, missing artifacts,
  mismatched settlement signatures, and raw-private-leaking transcript artifacts.
- Targeted contracts passed:
  `mvp_demo_three_agent_transcript_contract`,
  `mvp_demo_three_agent_claim_contract`,
  `mvp_demo_agent_harness_claim_contract`, `mvp_demo_claim_contract`, and
  `mvp_evaluator_demo_runbook_contract`.
- `cargo fmt --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed with `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
  CARGO_INCREMENTAL=0`.
- `make check` passed with the same cargo environment.
- Local `make demo-mvp` passed, canonical verifier passed, and the report had
  no `three_agent_transcript`, settlement claim, or three-agent claim in
  local-only mode.
- Devnet-required `make demo-mvp` passed with run
  `run-44206-1783563177548`, finalized Solana devnet signature
  `3e4VsMq6oUAwQMeFMw3N9zzyprkDtMZDNhT8BSybmzsnTAfnP3k1zUYjSfodmVV4eW891siPXyPTapQ62vJbEzx6`,
  and `three_agent_transcript` at
  `.kamn/demo/run-44206-1783563177548/proof/three-agent-transcript.json`.
- The transcript artifact recorded Agent A/B/C steps, participant-private versus
  restricted-public views, the devnet signature, `1000000` lamports, payer,
  recipient, finalized commitment, matching transcript digest, and no
  `raw_private_payload`.
- Report Markdown included `## Three-Agent View Boundary`, the transcript
  artifact path, and raw-private-payload redaction text.
- Pi `openai-codex/gpt-5.5` extraction passed and wrote
  `/tmp/kamn-pi-three-agent-transcript-evidence.json` with
  `three_agent_boundary.claim_status:"PASS"` and
  `claim_label:"devnet-backed"`.

## Deviations

- None.
