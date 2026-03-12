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

## Final evidence
- `cargo test -p kamn-node --test service_api_endpoint_auth_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::auth_scope_contract_tests::route_scope_policy_contract_tests::route_authz_contract_tests::integration_service_api_endpoint_route_authz_matrix_rejects_protected_paths_without_headers -- --exact --nocapture`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::ingress_guard_lifecycle_contract_tests::replay_guard_contract_tests::regression_service_api_endpoint_rejects_replayed_request_nonce_for_sender -- --exact --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-clean-20260312-101857-auth --base-ref origin/main --output-json /tmp/6928-touched-size-refactor4.json`
- touched-Rust result: `policy_decision=GO`

## Deviations
- Clean-clone validation used the Python touched-Rust entrypoint directly instead of the shell wrapper so the repo root stayed pinned to this disposable clone.
