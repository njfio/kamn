# Plan: #5674 Remove Remaining Agent-Lib Stubs via Service/SDK Route Expansion

## Approach
1. Add RED conformance tests first in `kamn-node` service API tests for the new routes and route-matrix counts.
2. Implement node route constants, payload handlers, path-id helpers, and auth scope mapping updates.
3. Add RED SDK tests for the four new client methods; implement typed methods and parsers.
4. Add RED agent-lib tests for former stub operations; implement SDK-backed operation methods.
5. Run targeted regression suites for `kamn-node`, `kamn-sdk`, `kamn-agent-lib` plus fmt/clippy.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `fixtures/runtime/service_api_scope_policy_fixture_matrix.txt`
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/src/lib.rs`
- `crates/kamn-sdk/tests/service_api_client.rs`
- `crates/kamn-agent-lib/src/client.rs`
- `crates/kamn-agent-lib/src/lib.rs`
- `crates/kamn-agent-lib/tests/*` (new/updated)

## Risks and Mitigations
- Risk: route-count and governance fixture drift breaks high-sensitivity tests.
- Mitigation: update matrix/fixture/constants in the same commit where routes are introduced and validate with targeted node tests.
- Risk: route path contract mismatch between node and SDK.
- Mitigation: explicit route composition tests in SDK harness using deterministic local server responses.
- Risk: broad touched surface can destabilize CI.
- Mitigation: strict crate-scoped test progression and atomic commit decomposition.

## Interfaces / Contracts
- New service payload contracts (deterministic):
  - `TaskAccept`: `{ "task_id": "...", "state": "accepted" }`
  - `TaskComplete`: `{ "task_id": "...", "state": "completed" }`
  - `EscrowFund`: `{ "escrow_id": "...", "state": "funded" }`
  - `EscrowRelease`: `{ "escrow_id": "...", "state": "released" }`
- SDK methods mirror these payloads with typed structs.
- Agent-lib methods return new typed structs and remove unsupported placeholder errors.

## ADR
- Not required; route additions follow existing service API architecture.
