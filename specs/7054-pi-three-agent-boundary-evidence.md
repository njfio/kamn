# 7054 Pi Three-Agent Boundary Evidence

## Objective

Bind the project-local Pi agent-harness evidence artifact to the richer MVP
three-agent escrow disclosure proof. A Pi-driven harness pass should prove that
the agent tools inspected the generated report's `three_agent_escrow_verification`
claim and preserved the same participant/private versus verifier/public
boundary in its evidence.

## Inputs/Outputs

Inputs:
- `.kamn/demo/latest/proof/report.json` or an explicit report path supplied to
  the Pi extension tools.
- The report's `three_agent_escrow_verification` claim.
- Existing optional `KAMN_MVP_AGENT_HARNESS_EVIDENCE` or CLI-provided agent
  harness evidence path.

Outputs:
- Pi-generated `kamn.mvp.agent-harness-evidence.v1` artifact with a
  `three_agent_boundary` block derived from the report.
- `verify-mvp-demo` compares the artifact's `three_agent_boundary` fields
  against the report being verified when `mcp_agent_harness_verification` is
  claimed.
- The evaluator runbook states that Pi evidence is bound to the three-agent
  disclosure proof without claiming generic Pi MCP protocol support.

## Boundaries/Non-goals

- No new exchange, escrow, settlement, bridge, or service architecture.
- No new npm or Rust dependencies.
- No production-readiness, mainnet, generalized privacy, or real economic value
  claim.
- No generic Pi MCP protocol claim beyond project-local Pi extension tools.
- No secret reads from `.kamn/devnet`, private key files, OAuth material, `.env`,
  or similar paths.
- No weakening of claim labels, dry-run/placeholder rejection, clippy,
  formatting, `make check`, or proof verifier semantics.

## Failure Modes

- Agent-harness evidence omits `three_agent_boundary`.
- Agent-harness evidence claims a `three_agent_escrow_verification` status or
  label that differs from the report being verified.
- Participant private field counts are zero or not greater than the verifier
  private field count.
- Verifier private field count is nonzero.
- A verifier private-view digest is present or claimed visible.
- Private payload redaction is missing or false.
- Report-derived `three_agent_boundary` fields are malformed or absent.
- Pi extension reads a secret-like path or silently writes evidence from a
  malformed report.

## Acceptance Criteria

- [ ] Pi-generated evidence records `three_agent_boundary.claim_status` from the
  report's `three_agent_escrow_verification` claim.
- [ ] Pi-generated evidence records `three_agent_boundary.claim_label` from the
  report's `three_agent_escrow_verification` claim.
- [ ] Pi-generated evidence records Agent A and Agent B private field counts,
  verifier private field count, private payload redaction, and verifier private
  digest visibility.
- [ ] `verify-mvp-demo` rejects agent-harness evidence that omits
  `three_agent_boundary` when `mcp_agent_harness_verification` is present.
- [ ] `verify-mvp-demo` rejects evidence where `claim_status` or `claim_label`
  conflicts with the report's three-agent claim.
- [ ] `verify-mvp-demo` rejects evidence where participant private counts are
  zero, verifier private count is nonzero, private payload redaction is false,
  or verifier private digest visibility is true.
- [ ] Existing valid `mcp-tools` and `pi-extension-tools` evidence remains
  accepted once it includes the three-agent boundary block.
- [ ] The runbook explains the Pi proof boundary and still states this is not a
  generic Pi MCP protocol proof.

## Files to Touch

- `.pi/extensions/kamn-mvp/index.ts`
- `crates/kamn-e2e-harness/src/mvp_demo/agent_harness.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract/support.rs`
- `docs/validation/mvp-evaluator-demo.md`

## Error Semantics

- Rust verifier failures must return explicit `Err(String)` messages naming the
  violated agent-harness or three-agent boundary field.
- Missing or mismatched evidence fails closed; no fallback may turn an invalid
  harness artifact into a passing local-only result.
- Pi tools must throw on malformed reports, missing three-agent claims, unsafe
  paths, failed verifier commands, and failed demo commands.
- Settlement and asset-movement claims remain devnet-backed only when the proof
  report includes valid devnet evidence.

## Test Plan

Red:
- Add negative verifier tests for missing `three_agent_boundary`, mismatched
  three-agent status/label, zero participant private counts, nonzero verifier
  private count, verifier private digest visibility, and missing redaction.
- Add/update the Pi extension marker contract to require `three_agent_boundary`
  evidence writing.

Green:
- Extend Pi evidence writing to extract the three-agent boundary from the report.
- Extend agent-harness verifier code to compare artifact fields against the
  report's `three_agent_escrow_verification` claim.
- Keep `mcp-tools` and `pi-extension-tools` execution surfaces accepted when the
  richer evidence is present.

Refactor:
- Keep verifier helpers small, single-purpose, and aligned with the existing
  flat JSON marker/extractor style.
- Avoid adding dependencies or broad JSON parsing architecture unless the
  existing verifier helpers cannot express the field checks safely.

Integration:
- Run targeted agent-harness and three-agent contract tests.
- Run `make demo-mvp` with a Pi-compatible evidence artifact and verify the
  generated report.
- Run formatting, strict clippy, and `make check`.

## Completion Evidence

- `cargo test -p kamn-e2e-harness --test mvp_demo_agent_harness_claim_contract -- --nocapture`
- `cargo test -p kamn-e2e-harness --test mvp_demo_three_agent_claim_contract -- --nocapture`
- `cargo test -p kamn-e2e-harness --test mvp_evaluator_demo_runbook_contract -- --nocapture`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `make check`
- `make demo-mvp`
- `cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json`
- Pi extension smoke or explicit blocker evidence if local Pi auth/runtime is
  unavailable.

## Deviations

- None yet.
