# Plan: Issue #5859

## Approach
1. Add a helper in `service_api_endpoint/server.rs` to resolve state-file path with precedence:
   - explicit `KAMN_SERVICE_API_STATE_FILE` when present,
   - otherwise deterministic default derived from role + bind address.
2. Wire `serve_service_api_endpoint_async` to use the new resolver and continue loading `ServiceApiMessageStore` from that path.
3. Add/adjust tests:
   - integration restart-persistence test that runs without explicit `KAMN_SERVICE_API_STATE_FILE`.
   - unit tests for resolution precedence and deterministic fallback shape.
4. Run targeted `kamn-node` tests plus `fmt`/`clippy` verification.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/server.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/5859/spec.md`
- `specs/5859/plan.md`
- `specs/5859/tasks.md`

## Risks & Mitigations
- Risk: deterministic default file path could cause test/state collisions.
  - Mitigation: derive from bind address + role; integration tests clean up generated file.
- Risk: behavior drift for explicit env configuration.
  - Mitigation: unit test explicit override precedence.
- Risk: filesystem write failures from invalid path construction.
  - Mitigation: sanitize path components and keep default under `std::env::temp_dir()`.

## Interfaces / Contracts
- Service API state-file resolution contract:
  - `KAMN_SERVICE_API_STATE_FILE` set -> use explicit value.
  - unset -> use deterministic default path from endpoint identity.
- Existing response payload contracts for message send/get remain unchanged.

## ADR
- Not required (local runtime behavior hardening; no dependency/protocol architecture decision).
