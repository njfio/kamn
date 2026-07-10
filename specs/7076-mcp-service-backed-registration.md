# Issue 7076: MCP Service-Backed Agent Registration

## Objective

Make the existing MCP `register` tool perform authenticated, durable agent
registration through KAMN's real `POST /v1/agents/register` service route
instead of returning an identity-only synthetic response.

## Inputs/Outputs

- Input: the existing empty MCP `register` arguments object.
- Input: the MCP process identity and signing key configured at startup.
- Input: deterministic MVP metadata: `agent_type=autonomous`,
  `model_family=mcp-agent`, and `capabilities=[mcp]`.
- Output: a successful MCP envelope containing the persisted DID, agent type,
  model family, capabilities, and reputation score returned by the service.
- Side effect: the authenticated sender profile is persisted by the existing
  service registration route.

## Boundaries/Non-goals

- Do not add or rename an MCP tool.
- Do not change the empty `register` input schema.
- Do not add configurable metadata or dependencies in this slice.
- Do not add Pi orchestration or claim a complete actor-driven workflow yet.
- Do not change task, escrow, settlement, Solana, or proof semantics.
- Do not fall back to identity-only success when the service route fails.

## Failure Modes

- Registration endpoint is unavailable.
- Service authentication or `agents:write` authorization fails.
- Service rejects invalid or conflicting metadata.
- Service returns malformed profile JSON.
- Returned DID does not match the authenticated MCP identity.
- Canonical registration payload used for signing differs from the HTTP body.

## Acceptance Criteria

- [ ] Real-backend MCP `register` invokes `POST /v1/agents/register`.
- [ ] The request is signed with `agents:write` and uses the canonical SDK
      registration payload.
- [ ] The persisted profile uses the deterministic MCP metadata contract.
- [ ] The MCP response exposes the persisted profile fields.
- [ ] Service failures and malformed responses fail hard with no fallback.
- [ ] Repeating identical registration remains compatible with the service's
      existing idempotent registration semantics.
- [ ] The `register` tool name and empty input schema do not change.
- [ ] Fake backends and existing task/escrow/proof tools remain compatible.

## Files To Touch

- `specs/7076-mcp-service-backed-registration.md`
- `crates/kamn-sdk/src/lib.rs`
- `crates/kamn-sdk/src/service_agent_registration.rs`
- `crates/kamn-sdk/src/service_client_bridge_misc_routes.rs`
- `crates/kamn-sdk/src/live/routes.rs`
- `crates/kamn-agent-lib/src/client.rs`
- `crates/kamn-agent-lib/src/lib.rs`
- `crates/kamn-mcp-server/src/lib.rs`
- `crates/kamn-mcp-server/src/registration.rs`
- `crates/kamn-mcp-server/src/dispatch.rs`
- `crates/kamn-mcp-server/tests/real_backend_integration_contract.rs`
- Existing SDK, agent-lib, and MCP regression tests as required.

## Error Semantics

- SDK metadata validation returns existing typed `SdkError::InvalidInput`.
- Agent-lib preserves SDK errors through `AgentLibError`; it does not log or
  substitute a local DID.
- MCP dispatch translates the propagated error into its existing structured
  failure envelope.
- A service-returned DID mismatch fails before MCP success is rendered.
- Network, authentication, conflict, and malformed-response failures remain
  distinguishable through existing SDK/agent-lib error mappings.

## Test Plan

- Red: real-backend MCP registration test requiring one observed service HTTP
  request and persisted profile metadata in the MCP response.
- Red: SDK canonical payload contract used by both auth signing and HTTP body.
- Green: extract the existing SDK registration payload builder, add agent-lib
  service registration, and wire MCP registration through it.
- Refactor: keep registration rendering in a dedicated MCP module and remove
  duplicate SDK metadata validation/payload code.
- Integration: run SDK registration tests, agent-lib contracts, MCP real
  backend/dispatch/protocol/inventory suites, formatter, strict workspace
  clippy, `make check`, and a live local MCP registration against KAMN runtime.

## Deviations

- None at specification time.

## Completion Evidence

- Pending implementation and verification.
