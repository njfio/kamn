## Objective
Add explicit regression coverage that the documented fail-closed DID policy is enforced at the
shared parser boundary and at service API auth ingress.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-types/tests/canonical_did_parse_integration.rs`
  - `crates/kamn-node/src/service_api_endpoint/auth.rs`
- Outputs:
  - parser regression tests that reject legacy `did:kamn:...` values with typed errors
  - auth regression coverage that rejects a request carrying a legacy sender DID header

## Boundaries/Non-goals
- Do not broaden the parser API surface.
- Do not add compatibility shims or auto-normalization.
- Do not change CI/workflow/shell surfaces.

## Failure modes
- Shared parser helpers accept `did:kamn:...` inputs or stop preserving typed prefix errors.
- Service API auth ingress accepts a legacy sender DID header or maps it to the wrong reason code.
- Tests only cover generic invalid prefixes and do not pin the legacy-shape regression.

## Acceptance criteria
- [x] `crates/kamn-types/tests/canonical_did_parse_integration.rs` explicitly rejects
      `did:kamn:agent:...` and `did:kamn:operator:...` with preserved typed errors.
- [x] `crates/kamn-node/src/service_api_endpoint/auth.rs` has unit coverage that a request with
      `x-kamn-sender-did: did:kamn:agent:...` fails with `REASON_CODE_AUTH_SENDER_DID_INVALID`.
- [x] No behavior change is required beyond what is needed to satisfy the red tests.
- [x] Focused parser/auth test commands pass locally.

## Files to touch
- `crates/kamn-types/tests/canonical_did_parse_integration.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `specs/6502-add-legacy-did-rejection-coverage.md`

## Error semantics
- Parser failures must continue returning the existing typed prefix errors.
- Service API auth ingress must continue mapping legacy sender DID rejection to
  `REASON_CODE_AUTH_SENDER_DID_INVALID`.

## Test plan
- Red:
  - add explicit regression assertions for legacy `did:kamn:...` parser rejection and service API
    sender-DID rejection
  - run the focused test commands and confirm they fail before implementation changes
- Green:
  - `cargo test -p kamn-types --test canonical_did_parse_integration -- --nocapture`
  - `cargo test -p kamn-node service_api_endpoint::auth::tests::regression_sender_did_header_rejects_legacy_did_shape -- --exact --nocapture`
- Refactor:
  - rerun the focused commands after any helper cleanup

## Deviations
- The shared parser helpers already rejected `did:kamn:...` with the correct typed prefix
  errors; no parser implementation change was required.
- The auth ingress work was limited to extracting the existing sender-DID header validation into a
  dedicated helper so the documented fail-closed behavior could be covered directly.

## Execution Evidence
- Red:
  - `cargo test -p kamn-types --test canonical_did_parse_integration -- --nocapture`
  - `cargo test -p kamn-node legacy_sender_did -- --nocapture`
- Green / Refactor / Integration:
  - `cargo test -p kamn-types --test canonical_did_parse_integration -- --nocapture`
  - `cargo test -p kamn-node service_api_endpoint::auth::tests::regression_sender_did_header_rejects_legacy_did_shape -- --exact --nocapture`
