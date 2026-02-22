# Plan: Issue #5799 — Align SDK/Agent-Lib Protected-Route Auth with Service Scope Policy

- Issue: #5799
- Spec: `specs/5799/spec.md`
- Status: Implemented
- Last Updated: 2026-02-22

## Implementation Approach
1. Add RED tests in `kamn-sdk` to assert protected-route scope header emission.
2. Add RED tests in `kamn-agent-lib` to assert chain-context override behavior.
3. Extend `ServiceRequestAuth` and service HTTP request builder to carry optional scope header.
4. Extend `ServiceApiHttpClient` to build auth with optional scope and configurable chain context.
5. Update `KamnAgentHandle` protected-route call sites to pass route-specific scope markers.
6. Run scoped crate tests (`kamn-sdk`, `kamn-agent-lib`, `kamn-cli`, `kamn-mcp-server`, `kamn-e2e-harness`) and capture live rerun evidence.
7. Update live `S-04` probe execution paths (sdk/cli/mcp) to satisfy replay/anti-spam policy in real service runtime.
8. Re-run live matrix and validate `S-04=PASS` across all modes.

## Affected Modules
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/tests/service_api_client.rs`
- `crates/kamn-agent-lib/src/client.rs`
- `crates/kamn-agent-lib/src/lib.rs`
- `crates/kamn-agent-lib/tests/*` (new/updated)
- `crates/kamn-e2e-harness/src/drivers/{sdk_direct,cli_scripted,mcp_agent}.rs`
- `docs/research/e2e-live-testing-prd-r55-live-probe-execution-evidence.md` (update with post-fix evidence)

## Risks / Mitigations
- Risk: Header/constructor changes break existing call sites.
  - Mitigation: Preserve backward-compatible constructor path and update all call sites with compile-time coverage.
- Risk: Scope mapping drift from node policy.
  - Mitigation: Mirror mapping from `service_api_endpoint/auth.rs` and codify via tests.
- Risk: Live rerun still fails due non-auth causes.
  - Mitigation: Adapt S-04 live probe execution strategy to avoid replay/anti-spam policy collisions and rerun matrix.

## Interfaces / Contracts
- Service auth header contract: `x-kamn-sender-did`, `x-kamn-request-nonce`, `x-kamn-request-signature`, `x-kamn-authz-scope`.
- Signature state-hash contract: `service-api:{chain_id}:{chain_version}`.

## ADR
- None required (contract-alignment fix, no architecture/dependency change).

## Execution Results
- Auth scope/header contract and chain-context override contract are implemented and covered by RED->GREEN tests.
- S-04 replay/anti-spam collisions were mitigated by per-step agent identity separation in SDK/CLI/MCP live probes.
- Live validation result: `S-04=PASS` across `sdk-direct`, `cli-scripted`, and `mcp-tau` modes against local API runtime.
