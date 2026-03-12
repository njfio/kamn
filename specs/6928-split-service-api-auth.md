# 6928-split-service-api-auth

## Objective
Split `crates/kamn-node/src/service_api_endpoint/auth.rs` into bounded, concern-based modules while preserving service API authentication behavior, fail-closed scope checks, replay protection, and sender anti-spam handling.

## Inputs/Outputs
- Inputs:
  - authenticated `ParsedRequest` values
  - `ServiceApiRuntimeState`
  - request headers, body, path, and method
  - replay-guard state and runtime anti-spam state
- Outputs:
  - unchanged auth/scope/anti-spam decisions
  - a thin root shell in `auth.rs`
  - bounded sibling modules for auth concerns
  - a hard-fail extraction contract for module layout and root budget

## Boundaries/Non-goals
- No changes to request-auth semantics
- No changes to public header names, reason codes, or route auth policy
- No new dependencies
- No weakening of existing tests or policies
- No unrelated service API refactors outside the auth surface

## Failure modes
- missing sender DID header remains fail-closed
- invalid sender DID remains fail-closed
- missing/invalid nonce remains fail-closed
- missing/invalid signer public key binding remains fail-closed
- signature verification failures remain fail-closed
- replay nonce detection remains fail-closed
- missing/mismatched scope remains fail-closed
- sender anti-spam limit violations remain fail-closed
- extraction contract fails if root shell or module layout regress

## Acceptance criteria
- [ ] `crates/kamn-node/src/service_api_endpoint/auth.rs` becomes a thin root shell under the staged root cap
- [ ] auth concerns are split into bounded modules with clear responsibilities
- [ ] a hard-fail extraction contract enforces the root shell and module layout
- [ ] existing auth-focused tests remain green without semantic drift
- [ ] touched-Rust size policy returns `policy_decision=GO`
- [ ] final spec records test evidence and any deviations

## Files to touch
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `crates/kamn-node/src/service_api_endpoint/auth/`
- `crates/kamn-node/tests/service_api_endpoint_auth_module_extraction_contract.rs`
- `specs/6928-split-service-api-auth.md`

## Error semantics
- Preserve existing typed fail-closed `RequestAuthFailure` and `ServiceApiReasonedError` behavior
- Preserve existing reason codes and error messages unless a test proves a correction is required
- Do not introduce silent fallback or relaxed auth paths

## Test plan
- Add a red extraction contract that fails while `auth.rs` is still monolithic
- Run the extraction contract green once the split is in place
- Run auth-focused real tests covering signature verification, scope enforcement, replay protection, and anti-spam paths
- Run touched-Rust size policy against the staged write set
