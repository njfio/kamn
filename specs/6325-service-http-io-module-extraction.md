# Spec: Issue 6325 - Extract service HTTP/IO helpers into dedicated module

## Objective
Extract HTTP/request IO helper routines from `crates/kamn-sdk/src/service.rs` into a dedicated internal module so `service.rs` remains focused on endpoint/client orchestration while preserving runtime behavior.

## Inputs/Outputs
- Inputs:
  - Existing helper functions in `service.rs` for endpoint authority parsing, request validation, auth header rendering, stream write/flush, and response read handling.
  - Existing constants and `SdkError` semantics used by those helpers.
- Outputs:
  - New module file under `crates/kamn-sdk/src/` containing extracted helper implementations.
  - `service.rs` module wiring/imports updated to call extracted helpers.
  - Extraction contract tests updated to assert module declaration and absence of inline helper definitions.

## Boundaries/Non-goals
- Do not change any public SDK API signatures.
- Do not change HTTP wire shape (headers, method/path formatting, body handling).
- Do not change error strings/codes surfaced to callers.
- Do not add dependencies.

## Failure modes
- Missing module wiring in `service.rs` causes compile/test failure.
- Behavioral drift in helper logic changes request validation or response read behavior.
- Test module imports fail if extracted helper visibility is not correctly scoped.

## Acceptance criteria (testable booleans)
- [x] `service.rs` declares `mod service_http_io;`.
- [x] `service.rs` no longer contains inline definitions for:
  - `write_and_flush_request`
  - `parse_host_port`
  - `normalize_route_segment`
  - `validate_http_header_value`
  - `validate_endpoint_host`
  - `validate_request_method`
  - `validate_request_path`
  - `render_auth_headers`
  - `read_response_bytes`
  - `read_response_text`
- [x] Existing `kamn-sdk` tests that exercise these helpers remain green.
- [x] `service_module_extraction_contract` test suite is green with added module/inline assertions.

## Files to touch
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/src/service_http_io.rs` (new)
- `crates/kamn-sdk/tests/service_module_extraction_contract.rs`
- `specs/6325-service-http-io-module-extraction.md`

## Error semantics
- Preserve hard-fail behavior and existing `SdkError` variants/messages.
- Preserve TLS-aware IO classification behavior currently used by response/request IO helpers.
- No silent fallback paths introduced.

## Test plan
- Red phase:
  - Extend `service_module_extraction_contract` with assertions for module wiring and inline helper removal; confirm failure before implementation.
- Green/refactor/integration phases:
  - `cargo test -p kamn-sdk --test service_module_extraction_contract`
  - `cargo test -p kamn-sdk --lib`
  - `cargo test -p kamn-sdk --test service_api_client`

## Phase 6 integration evidence
- `cargo test -p kamn-sdk --test service_module_extraction_contract`:
  - pass (`4 passed, 0 failed`)
- `cargo test -p kamn-sdk --lib`:
  - pass (`21 passed, 0 failed`)
- `cargo test -p kamn-sdk --test service_api_client`:
  - pass (`15 passed, 0 failed`)

## Deviations
- None.
