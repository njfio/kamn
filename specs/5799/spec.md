# Spec: Issue #5799 — Align SDK/Agent-Lib Protected-Route Auth with Service Scope Policy

- Issue: #5799
- Parent: #5797
- Milestone: `r52-e2e-live-runtime-integration-hardening`
- Priority: P1
- Status: Implemented
- Last Updated: 2026-02-22

## Problem Statement
Live `S-04` task-lifecycle probes fail against real `kamn-node` service API for two stacked reasons:
1. request-auth contract drift (signature state-hash mismatch + missing `x-kamn-authz-scope`), and
2. once auth is fixed, runtime replay/anti-spam gating still fails multi-step `S-04` execution in CLI/MCP/SDK live probes.

This blocks end-to-end live task lifecycle execution across sdk/cli/mcp modes.

## Scope
- Add optional auth scope support to SDK service request auth envelope and HTTP emission.
- Add protected-route scope assignment in `kamn-agent-lib` for task/escrow/message/channel/profile/query operations.
- Add chain context override support for agent-lib service auth signing so live clients can align with service chain config.
- Update live `S-04` probe execution strategy to avoid replay/anti-spam rejection under real service policy.
- Add tests validating scope header behavior and chain-context override behavior.

## Out of Scope
- Node-side auth policy changes.
- Runtime/Kolme protocol changes.
- CI workflow changes.

## Acceptance Criteria

### AC-1: Protected-route requests carry correct scope marker
Given agent-lib operations that call protected service routes,
When requests are built and sent,
Then each request includes `x-kamn-authz-scope` matching route policy (`messages:write|read`, `channels:write|read`, `tasks:write|read`, `escrow:write`, `agents:read`).

### AC-2: Client auth signature context can align with node chain context
Given node chain context differs from agent-lib defaults,
When chain context overrides are configured,
Then request signatures validate under service auth middleware instead of failing for signature mismatch.

### AC-3: Live S-04 auth blocker is eliminated
Given local `kamn-node` API is running,
When harness S-04 task lifecycle probes execute across sdk/cli/mcp with aligned chain context,
Then `S-04` transitions to `PASS` in all three modes without auth/replay/anti-spam rejection.

### AC-4: Regression coverage exists for new auth contract
Given the new auth behavior,
When tests run,
Then SDK/agent-lib tests verify scope header emission and chain-context override behavior.

## Conformance Cases

| ID | AC | Tier | Case |
|---|---|---|---|
| C-01 | AC-1 | Integration | Protected route request includes `x-kamn-authz-scope` with expected value in service client test harness. |
| C-02 | AC-2 | Unit | Agent-lib resolves chain-id/version overrides and uses them in request signature state-hash context. |
| C-03 | AC-3 | Functional | Live run against local `kamn-node` yields `S-04=PASS` across sdk/cli/mcp modes. |
| C-04 | AC-4 | Regression | Existing service auth tests remain green with new scope-aware request envelope. |
| C-05 | AC-3 | Regression | Live S-04 evidence and node logs show no replay/anti-spam rejection reason for probe path. |

## Success Metrics / Observable Signals
- Protected-route calls no longer return `service_api_auth_scope_header_missing`.
- Chain-aligned calls no longer return `service_api_auth_signature_verification_failed`.
- `S-04` live probe reaches `PASS` across sdk/cli/mcp paths.

## Implementation Evidence

- C-01 (AC-1): `cargo test -p kamn-sdk --test service_api_client` passes with scope-required route validation.
- C-02 (AC-2): `cargo test -p kamn-agent-lib --test service_auth_chain_context_contract` passes with chain override env (`KAMN_AGENT_CHAIN_ID`, `KAMN_AGENT_CHAIN_VERSION`).
- C-03/C-05 (AC-3): live matrix against local `kamn-node` API endpoint (`127.0.0.1:8080`) reports `S-01/S-04/S-06=PASS` for all three modes (`sdk-direct`, `cli-scripted`, `mcp-tau`).
- C-04 (AC-4): regression suites pass for `kamn-sdk`, `kamn-agent-lib`, `kamn-cli`, `kamn-mcp-server`, `kamn-e2e-harness`.
- Live evidence artifact: `docs/research/e2e-live-testing-prd-r55-live-probe-execution-evidence.md`.
