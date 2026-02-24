# Spec: Issue #5843 - Cryptographic Service Auth, Fail-Closed Live E2E, and Durable Message Delivery

- Issue: #5843
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
Current implementation has critical correctness and security gaps:
1. Service API request signatures are deterministic format strings (`sig:...:{body_len}`) and are forgeable.
2. E2E drivers mark scenarios `pass` when live probes are disabled, yielding false-positive scenario coverage.
3. Service API response routes are scaffolded and do not provide durable message state/delivery semantics across restart.
4. External execution mode does not enforce real runtime integration lifecycle gates for live validation.
5. Live workflow Kolme setup references a non-existent upstream binary shape (`kolme-node`) instead of a runnable local API profile.

## Scope
In scope:
- Replace service request auth verification with cryptographic signature verification bound to sender/nonce/state/payload.
- Update SDK/agent-lib auth signature generation to emit cryptographic signatures accepted by node middleware.
- Make E2E live scenario execution fail closed when live probes are not enabled.
- Add service API message persistence with restart-safe storage and recipient/channel delivery projection for live routes.
- Add runtime-orchestration contracts that require Kolme + KAMN runtime component startup evidence when external execution is requested.
- Define a repo-owned local Kolme setup profile for live tests that launches upstream Kolme API endpoints without modifying upstream source.

Out of scope:
- Changes to upstream `fpco/kolme` source.
- Full protocol redesign of all non-service transports.

## Acceptance Criteria
- AC-1: Service API auth signatures are cryptographic (not deterministic string-equality) and include payload-content binding; forged deterministic baseline signatures are rejected.
- AC-2: SDK/agent-lib service auth generation emits cryptographic request signatures compatible with node verification.
- AC-3: E2E scenario execution no longer returns `pass` when live probes are disabled; disabled-live path is fail-closed.
- AC-4: Service API message create/query/list routes persist message state with restart continuity and deterministic delivery projection.
- AC-5: External execution run contracts enforce runtime-orchestration preconditions for Kolme + KAMN components and expose deterministic pass/fail markers.
- AC-6: Live workflow Kolme bootstrap uses an upstream-supported runnable profile (`example-p2p api-server`) and deterministic health/readiness checks.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | deterministic legacy `sig:ed25519:baseline-v1:...` envelope | verification fails |
| C-02 | AC-1 | Unit/Functional | cryptographic signature over same sender/nonce/state/body | verification passes |
| C-03 | AC-2 | Functional | agent-lib `build_auth` and SDK request flow | authenticated route succeeds |
| C-04 | AC-3 | Regression | live toggle disabled with scenario `S-01` | status is `fail` (not `pass`) |
| C-05 | AC-4 | Integration | send message, restart endpoint with same state path, query message | message remains available |
| C-06 | AC-4 | Integration | send channel message then list channel messages | returned message list includes sent message id |
| C-07 | AC-5 | Functional | external execution requested with missing/invalid orchestration prerequisites | fail-closed deterministic error |
| C-08 | AC-5 | Functional | external execution requested with valid prerequisites | runtime orchestration markers include pass status |
| C-09 | AC-6 | Contract | CI workflow Kolme setup markers | uses `example-p2p` binary, `api-server` startup, and `/healthz` readiness |

## Test Mapping
- `cargo test -p kamn-core signature_profile -- --nocapture`
- `cargo test -p kamn-node service_api_endpoint_tests -- --nocapture`
- `cargo test -p kamn-sdk service_api_client -- --nocapture`
- `cargo test -p kamn-agent-lib auth_roundtrip -- --nocapture`
- `cargo test -p kamn-e2e-harness sdk_direct_live_toggle_contract cli_scripted_live_toggle_contract mcp_agent_live_toggle_contract -- --nocapture`
- `cargo test -p kamn-e2e-harness command_contract -- --nocapture`
- `cargo test -p kamn-e2e-harness --test phase4i_ci_workflow_contract -- --nocapture`

## Success Metrics / Observable Signals
- No service auth path accepts deterministic legacy signatures.
- Live-disabled scenarios cannot report `pass`.
- Service message state survives restart when state path is retained.
- External runtime execution reports deterministic component lifecycle pass/fail against real prerequisites.
- CI workflow Kolme setup launches a real upstream API profile and waits on `/healthz` before harness execution.
