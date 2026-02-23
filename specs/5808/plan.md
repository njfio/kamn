# Plan: Issue #5808 - S-02 Live Scenario Activation

- Issue: #5808
- Status: Completed
- Spec: `specs/5808/spec.md`

## Approach
1. Extend each driver's live probe storage and dispatch mapping to include explicit `S-02` probe/runner functions.
2. Implement `S-02` live probe helpers by reusing existing command/API helper pathways:
   - SDK-direct: `send_message` + `query_message` across distinct agent identities.
   - CLI-scripted: `send-message` + `query-message` command capture/field validation.
   - MCP-agent: `send_message` + `query_message` tool-call flow using framed JSON-RPC helper.
3. Add conformance tests first (RED) that assert `execute("S-02")` fails closed when injected S-02 probes fail.
4. Keep non-live and existing live scenario mappings stable to protect regression behavior.
5. Update milestone index active/completed markers for issue lifecycle closure.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `specs/5808/spec.md`
- `specs/5808/plan.md`
- `specs/5808/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: Scenario mapping drift causes `S-02` to stay non-live.
  - Mitigation: add direct `execute("S-02")` conformance tests in each driver.
- Risk: Probe parsing false positives from malformed command/tool payloads.
  - Mitigation: require non-empty `message_id` and validate query response `message_id` parity.
- Risk: Regression in current live-bound scenarios.
  - Mitigation: preserve existing `S-01/S-04/S-06` mappings and run full harness crate tests.

## Interfaces and Contracts
- No public API/wire-format schema changes.
- Internal driver contract change: `S-02` joins live-bound scenario set under opt-in live toggles.
- Fail-closed invariant preserved: any `S-02` live probe error maps to driver `status="fail"`.

## Verification Strategy
- RED/GREEN evidence via targeted `S-02` conformance tests in each driver module.
- Regression via existing live-toggle contract tests and full `kamn-e2e-harness` crate test lane.
