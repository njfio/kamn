# 7047 MCP Agent Harness Evidence In MVP Demo Proof

## Objective
Add an optional MVP proof-report bridge for agent-harness evaluation. The bridge
must let a local agent harness, including Pi when configured, drive or inspect
the KAMN MCP/demo path and leave durable evidence that the final proof report
was verified through the same claim boundaries as the direct harness path.

## Inputs/Outputs
Inputs:
- MVP demo proof report JSON emitted by `make demo-mvp`.
- Optional agent-harness evidence JSON emitted by a local MCP-agent evaluator.
- Existing MCP tool surface and MVP report verifier.

Outputs:
- Direct `make demo-mvp` remains valid without claiming agent-harness proof.
- When agent-harness proof is claimed, the report includes an explicit
  `mcp_agent_harness_verification` claim and an artifact path.
- `verify-mvp-demo --report ...` reads and validates the artifact when the
  report claims agent-harness verification.
- The verifier rejects mismatched, placeholder, or privacy-leaking
  agent-harness evidence.

## Boundaries/Non-goals
- Do not replace the existing MVP demo command or MCP live probe driver.
- Do not require Pi OAuth, OpenAI secrets, or committed credentials for Rust
  contract tests.
- Do not turn local MCP-agent evidence into settlement or asset-movement
  success; settlement remains devnet-backed only.
- Do not claim production readiness, mainnet settlement, or real economic value.

## Failure modes
- Report claims `mcp_agent_harness_verification` but omits an artifact path.
- Artifact path exists in the report but cannot be read by the verifier command.
- Artifact lacks MCP tool-surface markers for register, task, escrow, and proof
  verification.
- Artifact records verifier private view exposure.
- Artifact records dry-run, placeholder, or local-only settlement success.
- Artifact report path does not match the report being verified.
- Agent-harness claim uses an unsupported label or required placeholder/dry-run
  semantics.

## Acceptance criteria
- [ ] Direct local optional reports still verify without an agent-harness claim.
- [ ] Reports with a passing `mcp_agent_harness_verification` claim include an
  `agent_harness_evidence` artifact path.
- [ ] `verify-mvp-demo --report ...` rejects a passing agent-harness claim when
  the artifact path is missing or unreadable.
- [ ] `verify-mvp-demo --report ...` rejects agent-harness evidence where
  `verifier_private_view_visible` is true.
- [ ] `verify-mvp-demo --report ...` rejects agent-harness evidence that counts
  settlement without `settlement_claim_label=devnet-backed`.
- [ ] `verify-mvp-demo --report ...` accepts valid local MCP-agent evidence
  without adding any settlement claim to local-only reports.
- [ ] Existing claim-contract tests pass.

## Files to touch
- `crates/kamn-e2e-harness/src/mvp_demo/runner.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/report.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/report_artifacts.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/verify.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/verify_support.rs`
- Optional helper modules under `crates/kamn-e2e-harness/src/mvp_demo/`
- New focused contract test under `crates/kamn-e2e-harness/tests/`

## Error semantics
- Missing or unreadable claimed agent-harness evidence must hard-fail the
  `verify-mvp-demo` command.
- Privacy leaks must fail closed.
- Settlement claim-label mismatches must fail closed.
- Direct reports with no agent-harness claim must continue to verify normally.

## Test plan
Red:
- Add command-level tests for missing artifact, privacy leak, non-devnet
  settlement label, and valid MCP-agent evidence.
- Add pure report verifier coverage that rejects malformed
  `mcp_agent_harness_verification` claim shape.

Green:
- Add optional report artifact and claim rendering.
- Add command-level artifact validation for `verify-mvp-demo`.
- Keep the direct MVP command path unchanged when no evidence env is provided.

Refactor:
- Extract small agent-harness validation/rendering helpers so MVP verifier and
  report modules remain bounded and single-purpose.

Integration:
- Run targeted MVP claim-contract tests.
- Run `make demo-mvp`.
- Run the canonical report verifier against the latest report.
