# Plan: Issue #5828 - Live S-11 Signer-Rotation Activation

- Issue: #5828
- Spec: `specs/5828/spec.md`
- Status: Implemented

## Approach
1. Add RED fail-closed tests for `execute("S-11")` across sdk-direct, cli-scripted, and mcp-agent drivers.
2. Extend each driver's live-route map and probe wiring to include dedicated `S-11` signer-rotation probes.
3. Implement per-driver live `S-11` probes that verify:
   - baseline signer request acceptance,
   - rotated signer request acceptance,
   - stale/old signer replay rejection with deterministic reason marker validation.
4. Update live-toggle contracts so non-live-bound scenario checks move from `S-11` to `S-12`.
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
- `specs/5828/{spec,plan,tasks}.md`

## Interfaces / Contracts
- Driver execution contract: `execute("S-11")` must fail closed when live `S-11` probes error.
- New optional env inputs:
  - SDK: `KAMN_E2E_S11_PRIMARY_AGENT_NAME`, `KAMN_E2E_S11_ROTATED_AGENT_NAME`, `KAMN_E2E_S11_MESSAGE_PAYLOAD`, `KAMN_E2E_S11_ROTATED_MESSAGE_PAYLOAD`
  - CLI: `KAMN_E2E_S11_PRIMARY_AGENT_NAME`, `KAMN_E2E_S11_ROTATED_AGENT_NAME`, `KAMN_E2E_S11_MESSAGE_PAYLOAD`, `KAMN_E2E_S11_ROTATED_MESSAGE_PAYLOAD`
  - MCP: `KAMN_E2E_S11_PRIMARY_AGENT_NAME`, `KAMN_E2E_S11_ROTATED_AGENT_NAME`, `KAMN_E2E_S11_MESSAGE_PAYLOAD`, `KAMN_E2E_S11_ROTATED_MESSAGE_PAYLOAD`
- No protocol/wire-format changes.

## Risks and Mitigations
- Risk: replay-rejection semantics can be brittle if stale signer step does not validate deterministic reason markers.
  - Mitigation: validate explicit replay reason marker in all three probes.
- Risk: duplicate helper logic can weaken mutation resistance.
  - Mitigation: add explicit helper assertions and run in-diff mutation gate.
- Risk: top-level spec count regression can fail docs-contract tests.
  - Mitigation: include compensating archived-spec cleanup in same issue if count increases.

## ADR
- Not required: no architectural/dependency/protocol decision changes.
