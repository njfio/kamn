# Spec: Issue #6057 - Harden SDK service path/header injection validation

- Issue: #6057
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #6056

## Problem Statement
`kamn-sdk` assembles raw HTTP request frames. Although input validation exists, we need explicit fail-closed contracts for CRLF/control-byte injection vectors and endpoint base-path parsing so malformed path/header input is rejected deterministically before request emission.

## Scope
In scope:
- Add regression tests in `crates/kamn-sdk/src/service.rs` for route segment and auth-header CRLF/control-byte rejection.
- Enforce endpoint base-path validation during `ServiceApiClient::connect(...)` parsing.
- Preserve existing SDK public API signatures and request wire format.

Out of scope:
- Replacing transport stack with reqwest/ureq.
- Async SDK redesign.
- Server-side route/auth policy changes.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: `ServiceApiClient::connect(...)` fails when endpoint base-path contains invalid request-path characters.
- AC-2: `ServiceRequestAuth::new_with_scope(...)` rejects CR/LF/control-byte header values for signature and scope.
- AC-3: Route segment normalization rejects delimiter/control injection payloads (`/`, `?`, `#`, spaces, CRLF).
- AC-4: Existing SDK service tests remain green.

## Conformance Cases
- C-01 (Unit, AC-1): endpoint `http://127.0.0.1:8080/%0d%0a` handling remains path-safe; raw control-byte path input fails at connect.
- C-02 (Functional, AC-2): `ServiceRequestAuth::new_with_scope` with CRLF in signature returns `SdkError::InvalidInput`.
- C-03 (Functional, AC-2): `ServiceRequestAuth::new_with_scope` with CRLF in scope returns `SdkError::InvalidInput`.
- C-04 (Unit, AC-3): route-segment helpers reject disallowed delimiters and CRLF payloads.
- C-05 (Regression, AC-4): existing `kamn-sdk` service regression suite remains green.

## Success Metrics / Observable Signals
- New injection-shaped regression tests fail before parse hardening and pass after fix.
- Targeted `kamn-sdk` service tests pass locally and in CI fast gate.
