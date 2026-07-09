# 7064 Pi Actor Tool Receipts

## Objective

Make the Pi harness proof for devnet-backed three-agent reports actor-driven at
the local tool layer. The prior slice verifies `three_agent_actor_rehearsal`,
but Pi writes that rehearsal as one derived evidence object. This issue requires
Pi to collect actor-specific tool receipts before writing three-agent harness
evidence.

## Inputs/Outputs

Inputs:
- MVP proof report JSON.
- `three_agent_escrow_verification` claim when present.
- Agent A, Agent B, and Agent C verifier view artifact paths and digests from
  that report claim.
- In-process receipts created by project-local Pi extension actor tools.

Outputs:
- `three_agent_actor_tool_receipts` in Pi agent-harness evidence for
  devnet-backed three-agent reports.
- Receipts for:
  - Agent A registration.
  - Agent A transaction invocation.
  - Agent B registration.
  - Agent B task acceptance.
  - Agent C restricted-public proof verification.
- Verifier errors when required receipts are missing, malformed, mismatched,
  out of scope, or over-disclosing.

## Boundaries/Non-goals

- Do not build a production multi-agent runtime.
- Do not claim generic Pi MCP protocol support.
- Do not change Solana devnet settlement execution semantics.
- Do not add dependencies.
- Do not require actor tool receipts for local-only reports that do not claim
  `three_agent_escrow_verification`.
- Do not expose raw participant-private payloads or secrets.
- Do not claim mainnet, production readiness, broad exchange, generalized
  escrow, or real economic value.

## Failure Modes

- Pi writes three-agent harness evidence without first collecting all actor
  receipts.
- A receipt omits tool name, agent, action, sequence, report path, view scope,
  view artifact, view digest, or PASS outcome.
- A receipt binds to a different report path, view artifact, view digest, or
  scope than the verified report claim.
- Agent C receipt claims participant-private scope or includes participant
  private digest markers.
- Receipt evidence leaks raw private payload markers.
- Receipt evidence claims local-only, dry-run, placeholder, or absent
  settlement while the report claims devnet-backed three-agent success.

## Acceptance Criteria

- [x] Pi extension registers actor-specific tools for Agent A register, Agent A
  invoke transaction, Agent B register, Agent B accept task, and Agent C verify
  proof.
- [x] Each actor tool appends an in-process receipt with tool name, agent,
  action, sequence, report path, view scope, view artifact, view digest, and
  PASS outcome.
- [x] `kamn_write_agent_harness_evidence` fails loudly for devnet-backed
  `three_agent_escrow_verification` reports when required actor receipts were
  not collected first.
- [x] Pi-written harness evidence includes `three_agent_actor_tool_receipts`
  when required receipts exist.
- [x] `verify-mvp-demo` rejects missing, mismatched, over-disclosing,
  non-devnet, or raw-private-leaking receipt evidence.
- [x] Local-only reports remain valid with no actor receipts.
- [x] Evaluator docs describe the actor-tool Pi sequence and keep claim
  boundaries explicit.
- [x] Formatting, strict clippy, `make check`, targeted MVP contracts, local
  demo verifier, and devnet-required demo verifier remain green.

## Files To Touch

- `.pi/extensions/kamn-mvp/index.ts`
- `.pi/extensions/kamn-mvp/evidence.ts`
- `crates/kamn-e2e-harness/src/mvp_demo/agent_harness_actor_rehearsal.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract/*.rs`
- `docs/validation/mvp-evaluator-demo.md`

## Error Semantics

- Pi evidence writing returns a thrown error if a devnet-backed three-agent
  report is missing required in-process actor receipts.
- Verifier failures return explicit `Err(String)` messages; no fallback should
  turn missing receipt proof into success.
- Receipt validation compares artifact fields against the verified report claim
  and the top-level harness report path.
- Agent C participant-private scope, private digest exposure, nonzero private
  field evidence, or raw private payload markers fail verification.

## Test Plan

Red:
- Add verifier tests rejecting three-agent harness evidence without
  `three_agent_actor_tool_receipts`.
- Add verifier tests rejecting receipt view digest mismatch.
- Add verifier tests rejecting Agent C participant-private receipt scope or
  private digest exposure.
- Add Pi extension source contract tests requiring the actor-specific tool names
  and receipt marker.

Green:
- Add actor-specific Pi extension tools and in-process receipt collection.
- Extend Pi evidence writing to require receipts for devnet-backed three-agent
  reports and include them in the evidence artifact.
- Extend the Rust verifier to validate receipt shape and report binding.

Refactor:
- Keep receipt validation in small helpers.
- Reuse existing flat marker and extractor helpers; do not add a JSON
  dependency.
- Keep touched files under repo line-budget guidance.

Integration:
- Run the agent-harness contract tests.
- Run the three-agent view artifact, claim, transcript, command, and runbook
  contract tests.
- Run `cargo fmt --check`, strict workspace clippy, and `make check`.
- Run local `make demo-mvp` and canonical verifier.
- Run devnet-required `make demo-mvp` and canonical verifier, or record explicit
  external NO-GO evidence if devnet is unavailable.
- Run Pi with `openai-codex/gpt-5.5`, actor tools, evidence writing, and
  verifier tool against a devnet-backed report.

## Completion Evidence

- Red tests were committed before implementation:
  - verifier accepted three-agent harness evidence with no
    `three_agent_actor_tool_receipts`.
  - verifier accepted actor receipt view-digest mismatch.
  - verifier accepted Agent C receipt private-digest leakage.
  - Pi extension source lacked actor-specific tool names and receipt marker.
- Focused contract:
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo test -p kamn-e2e-harness --test mvp_demo_agent_harness_claim_contract -- --nocapture`
  - Result: 18 passed.
- Broader MVP contract matrix:
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo test -p kamn-e2e-harness --test mvp_demo_agent_harness_claim_contract --test mvp_demo_three_agent_view_artifact_contract --test mvp_demo_three_agent_claim_contract --test mvp_demo_three_agent_transcript_contract --test mvp_demo_claim_contract --test mvp_demo_command_contract --test mvp_evaluator_demo_runbook_contract -- --nocapture`
  - Result: 54 passed.
- Local gates:
  - `cargo fmt --check`
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 make check`
- Local demo:
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 make demo-mvp`
  - Result: `GO`, `devnet_mode=optional`, no settlement success claimed.
  - Canonical verifier passed for `.kamn/demo/latest/proof/report.json`.
- Devnet-backed demo:
  - `KAMN_MVP_DEVNET_MODE=required KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL=https://api.devnet.solana.com KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE=/Users/n/.config/solana/ted-dev.json KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY=BSN17KC1c5kUuA7ZaTXvMUnFbZUhizeaisYcAFeTsbEb KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS=1000000 KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT=finalized CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 make demo-mvp`
  - Result: `GO`, `devnet_settlement_asset_movement=devnet-backed`, `three_agent_escrow_verification=devnet-backed`.
  - Settlement signature: `2AcM7EGqVxkNkJQwxcG9AZv1weHWXdNHhjBomFUjAP5zCVu45bEsUaxJa1pnCZBrEc825nQVUe4s8G8aFXNa9Ere`.
  - `solana confirm -v --url https://api.devnet.solana.com 2AcM7EGqVxkNkJQwxcG9AZv1weHWXdNHhjBomFUjAP5zCVu45bEsUaxJa1pnCZBrEc825nQVUe4s8G8aFXNa9Ere`
  - Result: finalized transfer of 1,000,000 lamports.
- Pi actor-tool harness:
  - compact temp report: `/tmp/kamn-7064-pi-actor-receipt-report.json`.
  - evidence artifact: `/tmp/kamn-pi-7064-actor-receipt-evidence.json`.
  - `env -u OPENAI_API_KEY pi --model openai-codex/gpt-5.5 --extension .pi/extensions/kamn-mvp/index.ts --no-builtin-tools --tools kamn_inspect_mvp_report_boundaries,kamn_agent_a_register,kamn_agent_a_invoke_transaction,kamn_agent_b_register,kamn_agent_b_accept_task,kamn_agent_c_verify_three_agent_proof,kamn_write_agent_harness_evidence,kamn_verify_mvp_report --no-session -p "..."`
  - Result: Pi called all five actor tools in order, wrote receipt evidence, and `kamn_verify_mvp_report` passed.
  - Evidence inspection showed five ordered receipts: Agent A register,
    Agent A invoke transaction, Agent B register, Agent B accept task, and
    Agent C verify proof. Agent A/B receipts were `participant-private`; Agent C
    receipt was `restricted-public`.
