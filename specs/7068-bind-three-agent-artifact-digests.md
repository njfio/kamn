# 7068 Bind Three-Agent Artifact Digests

## Objective

Make the MVP three-agent proof bind transcript and per-agent view digest claims
to verifier-computed SHA-256 hashes over the generated artifact content. The
demo should continue to prove the same local/devnet story, but digest fields
must become tamper-evident instead of run-id marker strings.

## Inputs/Outputs

Inputs:
- MVP proof report JSON.
- `three_agent_escrow_verification` claim when present.
- Three-agent transcript artifact.
- Agent A, Agent B, and Agent C verifier view artifact files.

Outputs:
- Generated `sha256:<hex>` digests for `three_agent_transcript_digest`,
  `agent_a_view_digest`, `agent_b_view_digest`, and
  `agent_c_verifier_view_digest`.
- Verifier failures when transcript or view artifact content changes without a
  matching report claim update.
- Verifier failures when embedded digest fields do not match the
  self-reference-safe digest over their artifact payload.

## Boundaries/Non-goals

- Do not build new escrow, settlement, exchange, or runtime architecture.
- Do not change Solana devnet settlement execution or keypair handling.
- Do not broaden production readiness, mainnet, or asset-movement claims.
- Do not expose participant-private material to the Agent C verifier view.
- Do not weaken local-only/devnet-backed/dry-run/placeholder claim labels.
- Do not add a new third-party crate; reuse existing workspace hashing surface
  or existing workspace dependencies only.

## Failure Modes

- A generated report still uses marker strings such as
  `agent-a-view-digest-<run_id>`.
- `verify-mvp-demo` accepts a transcript artifact with changed content while
  `three_agent_transcript_digest` remains stale.
- `verify-mvp-demo` accepts an Agent A, Agent B, or Agent C verifier view
  artifact with changed content while its report digest remains stale.
- Hash computation includes the digest field value itself and becomes
  impossible to reproduce without self-reference ambiguity.
- View hash enforcement accidentally requires Agent C to expose participant
  private digests.

## Acceptance Criteria

- [x] `verify-mvp-demo` rejects transcript content tampering when the report's
  `three_agent_transcript_digest` remains stale.
- [x] `verify-mvp-demo` rejects Agent A view content tampering when
  `agent_a_view_digest` remains stale.
- [x] `verify-mvp-demo` rejects Agent B view content tampering when
  `agent_b_view_digest` remains stale.
- [x] `verify-mvp-demo` rejects Agent C verifier view content tampering when
  `agent_c_verifier_view_digest` remains stale.
- [x] Generated MVP reports write `sha256:<hex>` values for transcript and
  per-agent view digest claims.
- [x] Embedded transcript/view digest fields match the same recomputed digest
  values expected by the report claim.
- [x] Existing three-agent disclosure boundaries, devnet settlement evidence,
  Pi actor receipt evidence, local artifact binding, formatting, strict clippy,
  and `make check` remain green.

## Files To Touch

- `crates/kamn-e2e-harness/Cargo.toml`
- `crates/kamn-e2e-harness/src/mvp_demo/mod.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/artifact_digest.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/report.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/runner.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_claim.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_transcript.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_transcript_build.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_view_artifacts.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_views.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_three_agent_transcript_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_three_agent_view_artifact_contract.rs`
- `crates/kamn-e2e-harness/tests/support/three_agent_view_artifacts.rs`
- `specs/7068-bind-three-agent-artifact-digests.md`

## Error Semantics

- Missing or malformed digest fields fail closed with explicit `Err(String)`
  messages.
- Digest comparison must use recomputed artifact digests, not only report and
  artifact string equality.
- The digest canonical payload excludes the embedded digest field being checked
  (`transcript_digest` for the transcript and `view_digest` for a view) so the
  verifier can recompute the value deterministically.
- Any hash helper failure is a verifier failure. No fallback may convert an
  invalid digest artifact into a passing proof.

## Test Plan

Red:
- Add a transcript contract test that writes a valid report/transcript, appends
  an otherwise ignored field to the transcript, keeps the original report
  digest, and expects `verify-mvp-demo` to reject it.
- Add Agent A, Agent B, and Agent C verifier view contract tests that mutate
  otherwise ignored view content while keeping report digest claims stale and
  expect `verify-mvp-demo` to reject each artifact.
- Add generation contract coverage that requires the report digest claims to
  start with `sha256:`.

Green:
- Add a small artifact digest helper for SHA-256 tagged digesting over artifact
  JSON with one embedded digest field removed.
- Generate view artifacts with `view_digest` equal to the canonical view
  payload digest.
- Generate the transcript after view artifacts exist and set
  `transcript_digest` to the canonical transcript payload digest.
- Pass the computed digest values into the report claim instead of deriving
  marker strings from `run_id`.
- Recompute and compare transcript/view digests during verifier validation.

Refactor:
- Keep digest helpers isolated from field-specific verifier logic.
- Keep existing claim boundary checks and actor identity checks intact.
- Split validation helpers if touched files exceed repo line-budget guidance.

Integration:
- Run `mvp_demo_three_agent_transcript_contract`.
- Run `mvp_demo_three_agent_view_artifact_contract`.
- Run the broader MVP proof contract matrix covering claims, transcript, local
  artifacts, agent harness, command behavior, and evaluator runbook behavior.
- Run `cargo fmt --check`.
- Run strict workspace clippy.
- Run `make check`.
- Run local `make demo-mvp` and canonical `verify-mvp-demo`.
- Run the devnet-required demo and canonical verifier, or record explicit
  external NO-GO evidence if Solana devnet is unavailable.

## Completion Evidence

- Spec checkpoint:
  - Issue: #7068.
  - Spec: `specs/7068-bind-three-agent-artifact-digests.md`.
- Red tests:
  - `mvp_demo_command_contract::spec_c06_demo_mvp_devnet_required_records_settlement_evidence`
    failed because generated digest claims were not `sha256:` values.
  - `mvp_demo_three_agent_view_artifact_contract` failed stale digest tests for
    Agent A, Agent B, and Agent C verifier views because tampered artifacts
    still verified.
  - `mvp_demo_three_agent_transcript_contract::spec_c05_command_rejects_stale_transcript_digest_after_content_tamper`
    failed because a tampered transcript still verified.
- Green/refactor focused tests:
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
    CARGO_INCREMENTAL=0 cargo test -p kamn-e2e-harness --test
    mvp_demo_three_agent_view_artifact_contract --test
    mvp_demo_three_agent_view_digest_contract --test
    mvp_demo_three_agent_transcript_contract --test
    mvp_demo_command_contract -- --nocapture`
  - Result: 20 passed.
- Broader MVP proof matrix:
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
    CARGO_INCREMENTAL=0 cargo test -p kamn-e2e-harness --test
    mvp_demo_three_agent_view_artifact_contract --test
    mvp_demo_three_agent_view_digest_contract --test
    mvp_demo_three_agent_claim_contract --test
    mvp_demo_three_agent_transcript_contract --test
    mvp_demo_agent_harness_claim_contract --test
    mvp_demo_local_artifact_contract --test mvp_demo_claim_contract --test
    mvp_demo_command_contract --test mvp_evaluator_demo_runbook_contract --
    --nocapture`
  - Result: 64 passed.
- Local quality gates:
  - `cargo fmt --check`
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
    CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets
    --all-features -- -D warnings`
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
    CARGO_INCREMENTAL=0 make check`
- Local optional demo:
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
    CARGO_INCREMENTAL=0 make demo-mvp`
  - Result: `GO`, `devnet_mode=optional`, local-only MVP claims passed, and
    no devnet settlement or three-agent settlement claim was made.
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
    `2NrcVDZpuSSjveRdqbwnGj6SWCEUkzS8hZhNikSq5LNty882ydoK4Sa2ZWC99CYDfeTaXxXVGQjX24zWt9etnLsr`.
  - `solana confirm -v --url https://api.devnet.solana.com
    2NrcVDZpuSSjveRdqbwnGj6SWCEUkzS8hZhNikSq5LNty882ydoK4Sa2ZWC99CYDfeTaXxXVGQjX24zWt9etnLsr`
  - Result: finalized transfer of 1,000,000 lamports from
    `Ew2NpaFAK2TbUkbUMV54JN1gURSKkLWEypk5v9kJR7XU` to
    `BSN17KC1c5kUuA7ZaTXvMUnFbZUhizeaisYcAFeTsbEb`.
  - Canonical verifier passed for `.kamn/demo/latest/proof/report.json`.
  - Generated transcript and Agent A/B/C verifier view digest claims were
    `sha256:<hex>` values.
  - Generated Agent C verifier view reported `agent=agent_c_verifier`,
    `view_scope=restricted-public`, `private_field_count=0`, and no
    `participant_private_view_digest`.

## Deviations

- The spec focused on report, transcript, and view artifacts. During
  integration, the Pi/agent-harness actor rehearsal and actor receipt fixtures
  also had to move from marker digests to computed artifact digests so the
  broader MVP proof matrix stayed wired to the same verifier semantics.
