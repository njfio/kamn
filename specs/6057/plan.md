# Plan: Issue #6057

## Approach
1. Add RED tests in `crates/kamn-sdk/src/service.rs` (test module) for:
   - endpoint parse fail-closed on control-byte base-path values,
   - auth-header constructor rejection of CRLF signature/scope,
   - route-segment rejection of delimiter/control payloads.
2. Implement minimal parser hardening:
   - validate non-empty endpoint `base_path` via existing request-path validator during `ServiceEndpoint::parse`.
3. Keep changes local to sdk service module; avoid API/wire-format changes.
4. Run targeted format, clippy, and `kamn-sdk` service tests.

## Affected Modules
- `crates/kamn-sdk/src/service.rs`
- `specs/6057/spec.md`
- `specs/6057/plan.md`
- `specs/6057/tasks.md`

## Risks / Mitigations
- Risk: parse-time base-path validation could reject previously accepted endpoints.
  Mitigation: validation reuses existing request path rules already enforced at request time.
- Risk: regression noise from broad `kamn-sdk` test execution.
  Mitigation: run targeted test filters for service module and explicit regression names.

## Interfaces / Contracts
- Public API unchanged:
  - `ServiceApiClient::connect(endpoint: &str) -> Result<Self, SdkError>`
  - `ServiceRequestAuth::new_with_scope(...) -> Result<Self, SdkError>`
- Validation contract tightened:
  - invalid endpoint base-path fails at connect-time (not deferred to request-time).
