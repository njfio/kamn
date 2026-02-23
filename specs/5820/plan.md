# Plan: Issue #5820 - Live S-07 Replay-Protection Activation

- Issue: #5820
- Status: Completed
- Spec: `specs/5820/spec.md`

## Approach
1. Add RED fail-closed tests for `execute("S-07")` in sdk/cli/mcp driver test modules.
2. Extend each driver to include a dedicated `S-07` live probe/routing path.
3. Implement replay probes with mode-specific surfaces:
   - SDK-direct: two fresh handles using identical sender identity to force nonce replay.
   - CLI-scripted: two `send-message` invocations with identical agent identity; second must fail with replay reason marker.
   - MCP-agent: two `send_message` tool calls with identical agent identity; second tool response must be non-success and include replay reason marker.
4. Update toggle-contract tests to keep non-live scenario assertions coherent after `S-07` activation.
5. Run targeted tests, full harness regression, docs-contract guard, mutation gate, and workspace quality gate.
6. Finalize milestone/lifecycle artifacts.

## Affected Artifacts
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `crates/kamn-e2e-harness/tests/sdk_direct_live_toggle_contract.rs`
- `crates/kamn-e2e-harness/tests/cli_scripted_live_toggle_contract.rs`
- `specs/5820/spec.md`
- `specs/5820/plan.md`
- `specs/5820/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- `specs/archive/index.md`
- `specs/3964/ARCHIVED.md` (removed)
- `specs/archive/3964/plan.md` (removed)
- `specs/archive/3964/spec.md` (removed)
- `specs/archive/3964/tasks.md` (removed)

## Risks and Mitigations
- Risk: replay checks can flake if identity collisions from prior runs occur.
  - Mitigation: derive run-scoped S-07 agent identity suffix to keep first request unique while preserving same-sender replay within probe.
- Risk: CLI/MCP failure projections might hide reason-marker details.
  - Mitigation: capture and validate failure payload/stderr text containing `service_api_auth_replay_nonce_detected`.
- Risk: adding lifecycle artifacts breaches spec-dir cap guardrail.
  - Mitigation: run `review_r53_docs_contract`; if needed, offset with archived-pointer-only prune.

## Verification Strategy
- RED: targeted S-07 fail-closed tests fail before implementation.
- GREEN: targeted S-07 tests pass after routing + probe implementation.
- Regression: harness suite, docs-contract guard, in-diff mutants, and full workspace gate remain green.
