# Issue 6228 Plan

## Approach
1. Keep existing validation hooks in `service.rs` (`normalize_route_segment`, `validate_http_header_value`) as canonical enforcement points.
2. Add missing regression tests in `crates/kamn-sdk/tests/service_api_client.rs` for:
   - CRLF in DID route argument (`get_agent_profile`).
   - CRLF in auth scope header input.
3. Run targeted SDK test suite to verify fail-closed behavior and no regressions.

## Affected Modules
- `crates/kamn-sdk/tests/service_api_client.rs`
- `specs/6228/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: false confidence from partial route coverage.
  - Mitigation: cover a second route type (`did`) in addition to message_id.
- Risk: auth/header coverage gap.
  - Mitigation: explicit sender DID CRLF rejection test in `ServiceRequestAuth` constructor path.

## Interfaces
- No public API shape changes.
- Validation behavior remains `SdkError::InvalidInput` with deterministic field/reason pairs.
