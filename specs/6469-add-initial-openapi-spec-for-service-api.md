# Spec: Issue 6469 - Add initial OpenAPI spec for service API routes

## Objective
Publish an initial OpenAPI 3.1 document for the existing service API route
surface so external developers can discover endpoints and auth requirements from
a canonical machine-readable artifact.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-node/src/service_api_endpoint.rs` route constants and scope map
  - `docs/api/service-http-api.md` endpoint inventory
  - `crates/kamn-sdk/src/service_client_*` route contracts
- Outputs:
  - New OpenAPI spec file covering current service API endpoints.
  - Contract test asserting the spec includes required top-level markers and key
    route paths.

## Boundaries/Non-goals
- No API behavior changes.
- No endpoint additions/removals.
- No SDK code generation in this issue.

## Failure modes
- OpenAPI file omits existing routes.
- Auth header/security contract does not match service request-auth headers.
- File exists but is not versioned as OpenAPI 3.1 document.

## Acceptance criteria (testable booleans)
- [x] AC-1: OpenAPI 3.1 spec file exists for the service API surface.
- [x] AC-2: Spec includes key current paths:
      `/healthz`, `/v1/messages/send`, `/v1/messages/{id}`,
      `/v1/channels/create`, `/v1/channels/{id}/messages`,
      `/v1/tasks/create`, `/v1/tasks/{id}`, `/v1/tasks/{id}/accept`,
      `/v1/tasks/{id}/complete`, `/v1/escrow/fund`,
      `/v1/escrow/{id}/release`, `/v1/content/register`,
      `/v1/content/{id}`, `/v1/content/{id}/expire`,
      `/v1/content/{id}/tombstone`, `/v1/bridge/submit`,
      `/v1/bridge/{id}`, `/v1/bridge/{id}/forward`,
      `/v1/agents/{did}`, `/v1/events/ws`.
- [x] AC-3: Spec includes request-auth security/header contract markers for
      `X-KAMN-Sender-DID`, `X-KAMN-Request-Nonce`,
      `X-KAMN-Request-Signature`.
- [x] AC-4: Contract test validating OpenAPI markers and required paths passes.

## Files to touch
- `specs/6469-add-initial-openapi-spec-for-service-api.md`
- `docs/api/service-openapi.yaml`
- `crates/kamn-node/tests/service_api_endpoint_module_extraction_contract.rs`
- `docs/api/service-http-api.md`

## Error semantics
- Preserve existing runtime endpoint error semantics.
- Contract test failures must fail closed when required OpenAPI markers drift.

## Test plan
- Red:
  - Add/extend contract test to require OpenAPI file and required path/auth
    markers.
- Green:
  - Add OpenAPI file with required route/auth coverage.
  - Link docs inventory to OpenAPI artifact.
- Refactor:
  - Keep spec structure minimal and deterministic.
- Integration:
  - `cargo test -p kamn-node --test service_api_endpoint_module_extraction_contract`

## Phase 6 integration evidence
- OpenAPI artifact added at `docs/api/service-openapi.yaml` and linked from
  `docs/api/service-http-api.md`.
- Verified commands:
  - `cargo test -p kamn-node --test service_api_endpoint_module_extraction_contract`
  - `cargo fmt --all --check`

## Deviations
- None.
