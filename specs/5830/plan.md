# Plan: Issue #5830 - Live S-12 Retention/Deletion Activation

- Issue: #5830
- Spec: `specs/5830/spec.md`
- Status: Implemented

## Approach
1. Add RED fail-closed tests for `execute("S-12")` across sdk-direct, cli-scripted, and mcp-agent drivers.
2. Extend service API payload/auth scope contracts with deterministic content lifecycle routes needed by S-12 probes.
3. Add SDK + agent-lib wrappers for the new S-12 operations.
4. Add CLI commands and MCP tools/dispatch for S-12 operations used by live probes.
5. Extend each driver's live-route map and probe wiring to include dedicated `S-12` retention/deletion probes.
6. Run targeted RED->GREEN lanes, then package/workspace/docs-contract/mutation gates.
7. Apply compensating archived-spec cleanup if needed to preserve top-level `specs/` non-regression cap.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `fixtures/runtime/service_api_scope_policy_fixture_matrix.txt`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-agent-lib/src/lib.rs`
- `crates/kamn-cli/src/lib.rs`
- `crates/kamn-cli/src/commands/`
- `crates/kamn-mcp-server/src/tools.rs`
- `crates/kamn-mcp-server/src/dispatch.rs`
- `crates/kamn-e2e-harness/src/drivers/{sdk_direct,cli_scripted,mcp_agent}.rs`
- `crates/kamn-e2e-harness/tests/*_live_toggle_contract.rs`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- `specs/5830/{spec,plan,tasks}.md`

## Interfaces / Contracts
- New S-12 operation contracts (register/expire/tombstone/query) exposed consistently across service API, SDK, agent-lib, CLI, and MCP.
- Driver execution contract: `execute("S-12")` must fail closed when live S-12 probes error.
- Scope-policy contract includes deterministic `content:write` and `content:read` route mapping.

## Risks and Mitigations
- Risk: route/scope matrix drift due new protected routes.
  - Mitigation: update fixture + route auth matrix constants/tests in same slice.
- Risk: inventory drift for CLI subcommands and MCP tool registry.
  - Mitigation: add explicit contract tests and update deterministic inventory constants.
- Risk: top-level spec count regression can fail docs-contract tests.
  - Mitigation: include compensating archived-spec cleanup in same issue if count increases.

## ADR
- Not required: no new dependency or protocol framing changes.
