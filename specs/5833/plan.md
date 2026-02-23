# Plan: Issue #5833 - Live S-13 Bridge-Forwarding Activation

- Issue: #5833
- Spec: `specs/5833/spec.md`
- Status: Implemented

## Approach
1. Add RED fail-closed tests for `execute("S-13")` across sdk-direct, cli-scripted, and mcp-agent drivers.
2. Extend service API payload/auth scope contracts with deterministic bridge routes needed by S-13 probes.
3. Add SDK + agent-lib wrappers for the new S-13 operations.
4. Add CLI commands and MCP tools/dispatch for S-13 operations used by live probes.
5. Extend each driver's live-route map and probe wiring to include dedicated `S-13` bridge-forwarding probes.
6. Run targeted RED->GREEN lanes, then package/workspace/docs-contract/mutation gates.
7. Update milestone index and lifecycle markers in the same PR.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `fixtures/runtime/service_api_scope_policy_fixture_matrix.txt`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-kolme/src/service_api_scope.rs`
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-agent-lib/src/{client.rs,lib.rs}`
- `crates/kamn-cli/src/lib.rs`
- `crates/kamn-cli/src/commands/`
- `crates/kamn-cli/tests/command_activation_contract.rs`
- `crates/kamn-mcp-server/src/{tools.rs,dispatch.rs}`
- `crates/kamn-mcp-server/tests/*`
- `crates/kamn-e2e-harness/src/drivers/{sdk_direct,cli_scripted,mcp_agent}.rs`
- `crates/kamn-e2e-harness/tests/*_live_toggle_contract.rs`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- `specs/5833/{spec,plan,tasks}.md`

## Interfaces / Contracts
- New S-13 operation contracts (`submit_bridge_message`, `forward_bridge_message`, `query_bridge_message`) are exposed consistently across service API, SDK, agent-lib, CLI, and MCP.
- Driver execution contract: `execute("S-13")` must fail closed when live S-13 probes error.
- Scope-policy contract includes deterministic route mapping and fixture coverage for S-13 routes.

## Risks and Mitigations
- Risk: route/scope matrix drift due new protected routes.
  - Mitigation: update fixture + route auth matrix constants/tests in same slice.
- Risk: inventory drift for CLI subcommands and MCP tool registry.
  - Mitigation: update inventory constants and associated contract tests together.
- Risk: scenario routing regression in non-live mode.
  - Mitigation: keep live toggle tests and full harness regression lane in scope.

## ADR
- Not required: no new dependency or protocol framing changes.
