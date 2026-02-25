# Plan: Issue #5947 - Task: Restore service_api_endpoint root line-budget contract by removing redundant delegates

- Issue: #5947
- Spec: `specs/5947/spec.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Approach
1. RED evidence: run extraction contract to confirm root budget failure context (`935 > 900`) from CI signal.
2. Remove redundant root wrappers in `service_api_endpoint.rs` that forward to `auth`, `payload`, and `websocket` submodules.
3. Update `middleware_impl.rs` and `websocket.rs` to call submodule functions directly through `super::auth`, `super::payload`, and `super::websocket`.
4. Verify conformance and behavior parity with targeted extraction/route/websocket tests, then run formatting and strict clippy for touched crate.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/service_api_endpoint/websocket.rs`
- `crates/kamn-node/tests/service_api_endpoint_module_extraction_contract.rs` (verification only)

## Risks + Mitigations
- Risk: Accidental behavior changes while replacing wrapper call paths.
  - Mitigation: keep logic identical and only change call targets; run route/websocket regressions.
- Risk: Future reintroduction of wrapper inflation.
  - Mitigation: keep extraction contract gate in CI and maintain direct submodule calls.

## Interfaces / Contracts
- Primary contract source: `specs/5947/spec.md`.
- Gate contract: `crates/kamn-node/tests/service_api_endpoint_module_extraction_contract.rs`.
- No API/wire/protocol changes.

## ADR Requirement
- Not required (no architectural dependency/protocol decision change).
