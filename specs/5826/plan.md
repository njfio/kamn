# Plan: Issue #5826 - Live S-10 Topology-Coherence Activation

- Issue: #5826
- Spec: `specs/5826/spec.md`
- Status: Implemented

## Approach
1. Add RED fail-closed tests for `execute("S-10")` across sdk-direct, cli-scripted, and mcp-agent drivers.
2. Extend each driver's live-route map and probe wiring to include dedicated `S-10` topology-coherence probes.
3. Implement per-driver live `S-10` probes that verify:
   - primary-node send status validity,
   - secondary and tertiary node query status/id continuity for the same message,
   - secondary and tertiary node boundary health status validity.
4. Update live-toggle contracts so non-live-bound scenario checks move from `S-10` to `S-11`.
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
- `specs/5826/{spec,plan,tasks}.md`

## Interfaces / Contracts
- Driver execution contract: `execute("S-10")` must fail closed when live `S-10` probes error.
- New optional env inputs:
  - SDK: `KAMN_E2E_S10_PRIMARY_ENDPOINT`, `KAMN_E2E_S10_SECONDARY_ENDPOINT`, `KAMN_E2E_S10_TERTIARY_ENDPOINT`, `KAMN_E2E_S10_AGENT_NAME`, `KAMN_E2E_S10_MESSAGE_PAYLOAD`
  - CLI: `KAMN_E2E_S10_PRIMARY_ENDPOINT`, `KAMN_E2E_S10_SECONDARY_ENDPOINT`, `KAMN_E2E_S10_TERTIARY_ENDPOINT`, `KAMN_E2E_S10_AGENT_NAME`, `KAMN_E2E_S10_MESSAGE_PAYLOAD`
  - MCP: `KAMN_E2E_S10_PRIMARY_ENDPOINT`, `KAMN_E2E_S10_SECONDARY_ENDPOINT`, `KAMN_E2E_S10_TERTIARY_ENDPOINT`, `KAMN_E2E_S10_AGENT_NAME`, `KAMN_E2E_S10_MESSAGE_PAYLOAD`
- No protocol/wire-format changes.

## Risks and Mitigations
- Risk: secondary/tertiary endpoints may not be available in all environments.
  - Mitigation: default secondary/tertiary endpoints to primary endpoint while keeping fail-closed continuity assertions.
- Risk: duplicate helper logic can weaken mutation resistance.
  - Mitigation: add explicit helper assertions and run in-diff mutation gate.
- Risk: top-level spec count regression can fail docs-contract tests.
  - Mitigation: include compensating archived-spec cleanup in same issue if count increases.

## ADR
- Not required: no architectural/dependency/protocol decision changes.
