# Issue 6228 Spec

Status: Implemented
Priority: P0
Milestone: R59 Swarm Gap Closure
Parent: #6223

## Problem Statement
SDK HTTP request construction must fail closed against CRLF/header injection and unsafe route interpolation. Existing hardening in `kamn-sdk` needs issue-scoped conformance verification and explicit regression coverage for additional auth/header and DID path cases.

## Scope
In scope:
- Verify and retain route-segment validation before path interpolation.
- Verify and retain header-value validation before header interpolation.
- Add regression tests for CRLF injection attempts not yet explicitly covered.

Out of scope:
- Replacing SDK transport stack with a different HTTP client implementation.
- Non-SDK server-side request validation changes.

## Acceptance Criteria
- AC-1: SDK route interpolation rejects CR/LF and unsafe characters fail-closed.
- AC-2: SDK auth/header interpolation rejects CR/LF and invalid header bytes fail-closed.
- AC-3: Regression tests cover message-id route injection, DID route injection, and header/auth injection paths.
- AC-4: Targeted SDK tests pass.

## Conformance Cases
- C-01 (AC-1, Regression): `get_message()` rejects CRLF route payload before request emission.
- C-02 (AC-1, Regression): `get_agent_profile()` rejects CRLF DID route payload before request emission.
- C-03 (AC-2, Regression): `ServiceRequestAuth` rejects CRLF signature header payload.
- C-04 (AC-2, Regression): `ServiceRequestAuth` rejects CRLF scope header payload.
- C-05 (AC-4, Functional): `cargo test -p kamn-sdk service_api_client` passes.

## Success Metrics
- SDK rejects CRLF/path/header injection attempts locally with deterministic `SdkError::InvalidInput`.
- No regression in existing service API contract test suite.
