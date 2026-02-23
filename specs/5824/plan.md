# Plan: Issue #5824 - Live S-09 Transport-Failover Activation

- Issue: #5824
- Spec: `specs/5824/spec.md`
- Status: Implemented

## Approach
1. Add RED fail-closed tests for `execute("S-09")` across sdk-direct, cli-scripted, and mcp-agent drivers.
2. Extend each driver's live-route map and probe wiring to include dedicated `S-09` transport-failover probes.
3. Implement per-driver live `S-09` probes that verify:
   - pre-failover send/query status validity against primary endpoint,
   - failover-boundary health step validity against failover endpoint,
   - post-failover send/query status validity against failover endpoint,
   - distinct pre/post message IDs.
4. Update live-toggle contracts so non-live-bound scenario checks move from `S-09` to `S-10` where required.
5. Run targeted RED->GREEN test lanes, then full harness and docs-contract regression gates.
6. Apply compensating archived-spec cleanup if needed to preserve the top-level `specs/` non-regression cap.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `crates/kamn-e2e-harness/tests/sdk_direct_live_toggle_contract.rs`
- `crates/kamn-e2e-harness/tests/cli_scripted_live_toggle_contract.rs`
- `crates/kamn-e2e-harness/tests/mcp_agent_live_toggle_contract.rs`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- `specs/5824/{spec,plan,tasks}.md`

## Interfaces / Contracts
- Driver execution contract: `execute("S-09")` must fail closed when live `S-09` probes error.
- New optional env inputs:
  - SDK: `KAMN_E2E_S09_FAILOVER_ENDPOINT`, `KAMN_E2E_S09_AGENT_NAME`, `KAMN_E2E_S09_PRE_MESSAGE_PAYLOAD`, `KAMN_E2E_S09_POST_MESSAGE_PAYLOAD`
  - CLI: `KAMN_E2E_S09_FAILOVER_ENDPOINT`, `KAMN_E2E_S09_AGENT_NAME`, `KAMN_E2E_S09_PRE_MESSAGE_PAYLOAD`, `KAMN_E2E_S09_POST_MESSAGE_PAYLOAD`
  - MCP: `KAMN_E2E_S09_FAILOVER_ENDPOINT`, `KAMN_E2E_S09_AGENT_NAME`, `KAMN_E2E_S09_PRE_MESSAGE_PAYLOAD`, `KAMN_E2E_S09_POST_MESSAGE_PAYLOAD`
- No protocol/wire-format changes.

## Risks and Mitigations
- Risk: failover endpoint may not be available in all environments.
  - Mitigation: default failover endpoint to primary endpoint while keeping fail-closed continuity assertions.
- Risk: duplicate helper logic can weaken mutation resistance.
  - Mitigation: add explicit helper assertions and run in-diff mutation gate.
- Risk: top-level spec count regression can fail docs-contract tests.
  - Mitigation: include compensating archived-spec cleanup in same issue if count increases.

## ADR
- Not required: no architectural/dependency/protocol decision changes.
