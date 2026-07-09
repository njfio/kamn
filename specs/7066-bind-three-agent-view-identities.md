# 7066 Bind Three-Agent View Identities

## Objective

Make the MVP three-agent proof verifier bind every per-agent view artifact to
the exact actor identity it represents. The report, transcript, Pi actor
receipts, and view artifacts should all use one stable actor vocabulary:
`agent_a`, `agent_b`, and `agent_c_verifier`.

## Inputs/Outputs

Inputs:
- MVP proof report JSON.
- `three_agent_escrow_verification` claim when present.
- Three-agent transcript artifact.
- Agent A, Agent B, and Agent C verifier view artifact files.

Outputs:
- Verifier failures when an Agent A, Agent B, or Agent C verifier view artifact
  declares the wrong `agent` value.
- Generated MVP demo view artifacts whose `agent` fields match the report,
  transcript, and Pi actor receipt vocabulary.

## Boundaries/Non-goals

- Do not build a production multi-agent runtime or scheduler.
- Do not change Solana devnet settlement execution semantics.
- Do not change devnet keypair handling.
- Do not broaden production readiness, mainnet, generic exchange, or broad
  escrow claims.
- Do not add dependencies.
- Do not weaken existing transcript, local artifact, receipt, claim, demo, or
  verifier semantics.

## Failure Modes

- Agent A view artifact declares any `agent` other than `agent_a`.
- Agent B view artifact declares any `agent` other than `agent_b`.
- Agent C verifier view artifact declares any `agent` other than
  `agent_c_verifier`.
- Generated demo artifacts drift back to `agent_c` while the rest of the
  three-agent proof surface uses `agent_c_verifier`.
- Verifier continues to accept swapped participant view identities because the
  settlement and digest fields still match.

## Acceptance Criteria

- [x] `verify-mvp-demo` rejects an Agent A view artifact whose `agent` field is
  not `agent_a`.
- [x] `verify-mvp-demo` rejects an Agent B view artifact whose `agent` field is
  not `agent_b`.
- [x] `verify-mvp-demo` rejects the verifier view artifact unless its `agent`
  field is `agent_c_verifier`.
- [x] `make demo-mvp` writes `agent_c_verifier` inside the Agent C verifier
  view artifact.
- [x] Existing local-only reports, devnet-backed reports, Pi actor receipt
  evidence, transcript binding, local artifact binding, formatting, strict
  clippy, and `make check` remain green.

## Files To Touch

- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_views.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_view_artifacts.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_three_agent_view_artifact_contract.rs`
- `crates/kamn-e2e-harness/tests/support/three_agent_view_artifacts.rs`
- `specs/7066-bind-three-agent-view-identities.md`

## Error Semantics

- Missing or mismatched view artifact identities fail closed with explicit
  `Err(String)` messages.
- Identity validation must compare the artifact's `agent` field directly. It
  must not infer identity only from file path, claim field, digest string, or
  transcript path.
- Agent C verifier identity drift fails verification even if the view remains
  restricted-public.
- No fallback should convert a malformed view identity into a successful
  devnet-backed three-agent claim.

## Test Plan

Red:
- Add a verifier test rejecting an Agent A view artifact with a wrong `agent`
  field.
- Add a verifier test rejecting an Agent B view artifact with a wrong `agent`
  field.
- Add a verifier test rejecting an Agent C verifier view artifact with
  `agent_c` instead of `agent_c_verifier`.

Green:
- Require exact actor identity markers in participant and verifier view
  validation.
- Change generated Agent C verifier view artifacts to declare
  `agent_c_verifier`.
- Update fixtures to use the same identity vocabulary.

Refactor:
- Keep identity checks in small helpers beside existing view artifact
  validation.
- Reuse existing marker/extractor helpers.
- Keep touched Rust files under repo line-budget guidance.

Integration:
- Run `mvp_demo_three_agent_view_artifact_contract`.
- Run the broader MVP proof contract matrix covering claim, transcript, local
  artifact, agent harness, command, and evaluator runbook behavior.
- Run `cargo fmt --check`.
- Run strict workspace clippy.
- Run `make check`.
- Run local `make demo-mvp` and canonical `verify-mvp-demo`.
- Run the devnet-required demo and canonical verifier, or record explicit
  external NO-GO evidence if devnet is unavailable.

## Completion Evidence

- Red tests were committed before implementation:
  - `spec_c04_command_rejects_agent_a_view_identity_mismatch`.
  - `spec_c05_command_rejects_agent_b_view_identity_mismatch`.
  - `spec_c06_command_rejects_agent_c_short_identity`.
  - Focused red result:
    `mvp_demo_three_agent_view_artifact_contract` returned 3 passed and 3
    expected failures because the verifier accepted actor identity drift.
- Green/refactor focused evidence:
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
    CARGO_INCREMENTAL=0 cargo test -p kamn-e2e-harness --test
    mvp_demo_three_agent_view_artifact_contract -- --nocapture`
  - Result: 6 passed.
- Adjacent transcript and agent-harness contracts:
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
    CARGO_INCREMENTAL=0 cargo test -p kamn-e2e-harness --test
    mvp_demo_three_agent_transcript_contract --test
    mvp_demo_agent_harness_claim_contract -- --nocapture`
  - Result: 22 passed.
- Broader MVP contract matrix:
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
    CARGO_INCREMENTAL=0 cargo test -p kamn-e2e-harness --test
    mvp_demo_three_agent_view_artifact_contract --test
    mvp_demo_three_agent_claim_contract --test
    mvp_demo_three_agent_transcript_contract --test
    mvp_demo_agent_harness_claim_contract --test
    mvp_demo_local_artifact_contract --test mvp_demo_claim_contract --test
    mvp_demo_command_contract --test mvp_evaluator_demo_runbook_contract --
    --nocapture`
  - Result: 60 passed.
- Local gates:
  - `cargo fmt --check`
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
    CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets
    --all-features -- -D warnings`
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
    CARGO_INCREMENTAL=0 make check`
- Local demo:
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
    CARGO_INCREMENTAL=0 make demo-mvp`
  - Result: `GO`, `devnet_mode=optional`, local-only MVP claims passed, no
    settlement or three-agent devnet success claimed.
  - Canonical verifier passed for `.kamn/demo/latest/proof/report.json`.
- Devnet-backed demo:
  - `KAMN_MVP_DEVNET_MODE=required
    KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com
    KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL=https://api.devnet.solana.com
    KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE=/Users/n/.config/solana/ted-dev.json
    KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY=BSN17KC1c5kUuA7ZaTXvMUnFbZUhizeaisYcAFeTsbEb
    KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS=1000000
    KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT=finalized
    CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
    CARGO_INCREMENTAL=0 make demo-mvp`
  - Result: `GO`, `devnet_settlement_asset_movement=devnet-backed`, and
    `three_agent_escrow_verification=devnet-backed`.
  - Settlement signature:
    `4WjCeVjMbWN5tasBJHArvNgHGkym58ohYXMAsphoEz4KHNmDXyJ2pVqhUQWuVDjYnmPxFJ8Ds6LpnvtZwZWiXfwG`.
  - `solana confirm -v --url https://api.devnet.solana.com
    4WjCeVjMbWN5tasBJHArvNgHGkym58ohYXMAsphoEz4KHNmDXyJ2pVqhUQWuVDjYnmPxFJ8Ds6LpnvtZwZWiXfwG`
  - Result: finalized transfer of 1,000,000 lamports.
  - Canonical verifier passed for `.kamn/demo/latest/proof/report.json`.
  - Generated `agent-c-verifier-view.json` reported
    `agent=agent_c_verifier`, `view_scope=restricted-public`, and
    `private_field_count=0`.
  - Generated `three-agent-transcript.json` reported
    `agent_c_verifier: restricted-public` and
    `agent_c_verifier_verified`.
- Pi actor-tool harness:
  - `env -u OPENAI_API_KEY pi --model openai-codex/gpt-5.5 --extension
    .pi/extensions/kamn-mvp/index.ts --no-builtin-tools --tools
    kamn_inspect_mvp_report_boundaries,kamn_agent_a_register,kamn_agent_a_invoke_transaction,kamn_agent_b_register,kamn_agent_b_accept_task,kamn_agent_c_verify_three_agent_proof,kamn_write_agent_harness_evidence,kamn_verify_mvp_report
    --no-session -p "..."`
  - Result: Pi wrote `/tmp/kamn-pi-7066-actor-identity-evidence.json`,
    recorded five ordered actor receipts, and reported verifier pass.
  - Evidence inspection showed Agent C receipt as
    `kamn_agent_c_verify_three_agent_proof`, `agent_c_verifier`,
    `restricted-public`.

## Deviations

- The spec originally focused on view artifacts. During integration, the
  transcript step and view-map vocabulary were also changed from `agent_c` to
  `agent_c_verifier` so report, transcript, view artifact, and Pi receipt
  actor labels stay aligned.
