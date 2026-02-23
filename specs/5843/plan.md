# Plan: Issue #5843

## Approach
1. Implement cryptographic service-auth profile in `kamn-core::signature_profile` with canonical message signing/verification helpers.
2. Wire node service-auth middleware to cryptographic verification path and reject legacy deterministic signatures.
3. Update SDK/agent-lib auth generation to produce cryptographic signatures from configured signing material.
4. Introduce service API runtime message store with optional file persistence and wire send/query/list handlers through it.
5. Harden E2E drivers to fail closed when live probes are disabled.
6. Harden run-contract external execution markers to enforce real orchestration preconditions.
7. Define and wire a local Kolme live setup profile in workflow using an upstream-supported runnable binary.

## Affected Modules
- `crates/kamn-core/src/signature_profile.rs`
- `crates/kamn-node/src/service_api_endpoint/{auth.rs,middleware_impl.rs,payload.rs,server.rs}`
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-agent-lib/src/client.rs`
- `crates/kamn-e2e-harness/src/drivers/{sdk_direct.rs,cli_scripted.rs,mcp_agent.rs}`
- `crates/kamn-e2e-harness/src/run_contract.rs`
- `.github/workflows/e2e-live.yml`
- `crates/kamn-e2e-harness/tests/phase4i_ci_workflow_contract.rs`

## Risks and Mitigations
- Risk: breaking existing deterministic auth tests.
  - Mitigation: add explicit crypto test helpers and update endpoint tests to use new signing function.
- Risk: message persistence introduces flakiness in parallel tests.
  - Mitigation: isolate persistence path using per-test temporary files.
- Risk: external execution contract changes break scaffold expectations.
  - Mitigation: keep deterministic markers while enforcing fail-closed prerequisites and update contract tests accordingly.
- Risk: upstream Kolme binary entrypoint drift breaks live workflow startup.
  - Mitigation: pin workflow to an upstream-supported runnable profile (`example-p2p api-server`) and enforce marker checks in workflow contract tests.

## Interface/Contract Changes
- Service auth signature profile moves from deterministic baseline matcher to cryptographic verifier.
- E2E live-disabled execution status contract changes from `pass` to fail-closed.
- Service API runtime state gains message persistence store.
- Live workflow Kolme bootstrap contract changes from `kolme-node` assumption to `example-p2p api-server` + `/healthz` readiness.

## ADR
- No new ADR file required for this scoped hardening wave; no new external dependency family introduced.
