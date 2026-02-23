# Plan: Issue #5822 - Live S-08 Node-Crash-Recovery Activation

- Issue: #5822
- Spec: `specs/5822/spec.md`
- Status: Completed

## Approach
1. Add RED fail-closed tests for `execute("S-08")` across sdk-direct, cli-scripted, and mcp-agent drivers.
2. Extend each driver's live-route map and probe wiring to include dedicated `S-08` continuity probes.
3. Implement per-driver live `S-08` probes that verify:
   - pre-boundary send/query status validity,
   - recovery-boundary health step validity,
   - post-boundary send/query status validity,
   - distinct pre/post message IDs.
4. Update live-toggle contracts so non-live-bound scenario checks move from `S-08` to `S-09` where required.
5. Run targeted RED->GREEN test lanes, then full harness and docs-contract regression gates.
6. Apply compensating archived-spec cleanup if needed to preserve the top-level `specs/` non-regression cap.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `crates/kamn-e2e-harness/tests/sdk_direct_live_toggle_contract.rs`
- `crates/kamn-e2e-harness/tests/cli_scripted_live_toggle_contract.rs`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- `specs/5822/{spec,plan,tasks}.md`
- `specs/archive/index.md`
- `specs/3965/ARCHIVED.md` (removed)
- `specs/archive/3965/plan.md` (removed)
- `specs/archive/3965/spec.md` (removed)
- `specs/archive/3965/tasks.md` (removed)

## Interfaces / Contracts
- Driver execution contract: `execute("S-08")` must fail closed when live `S-08` probes error.
- New optional env inputs:
  - SDK: `KAMN_E2E_S08_AGENT_NAME`, `KAMN_E2E_S08_PRE_MESSAGE_PAYLOAD`, `KAMN_E2E_S08_POST_MESSAGE_PAYLOAD`
  - CLI: `KAMN_E2E_S08_AGENT_NAME`, `KAMN_E2E_S08_PRE_MESSAGE_PAYLOAD`, `KAMN_E2E_S08_POST_MESSAGE_PAYLOAD`
  - MCP: `KAMN_E2E_S08_AGENT_NAME`, `KAMN_E2E_S08_PRE_MESSAGE_PAYLOAD`, `KAMN_E2E_S08_POST_MESSAGE_PAYLOAD`
- No protocol/wire-format changes.

## Risks and Mitigations
- Risk: live environments may not provide deterministic recovery boundary behavior.
  - Mitigation: enforce fail-closed checks on status/ID continuity and explicit health-step success markers.
- Risk: duplicate helper logic can weaken mutation resistance.
  - Mitigation: add explicit helper assertions and run in-diff mutation gate.
- Risk: top-level spec count regression can fail docs-contract tests.
  - Mitigation: include compensating archived-spec cleanup in same issue if count increases.

## ADR
- Not required: no architectural/dependency/protocol decision changes.
