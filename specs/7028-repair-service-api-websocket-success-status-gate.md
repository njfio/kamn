# 7028-repair-service-api-websocket-success-status-gate

## Objective
Restore the Fast Gate service API websocket live validation by aligning the
standalone websocket validator's authenticated upgrade request with the current
fail-closed service API auth contract.

## Inputs/Outputs
- Inputs:
  - `scripts/runtime/validate_service_api_websocket_live.sh`
  - `scripts/runtime/test_validate_service_api_websocket_live.sh`
  - `scripts/runtime/validate_service_api_axum_ingress_live.sh`
  - `crates/kamn-node/src/service_api_endpoint/server.rs`
- Outputs:
  - The focused websocket validator sends explicit signer public key evidence on
    authenticated websocket upgrade requests.
  - The validator still fails closed for invalid websocket version, missing
    upgrade headers, and missing auth headers.
  - Fast Gate no longer fails with `expected websocket success status line`.

## Boundaries/Non-goals
- Do not weaken websocket upgrade/auth validation.
- Do not count dry-run output as live websocket success.
- Do not bypass Fast Gate, remove websocket checks, or broaden this into MVP
  demo feature work.
- Do not add committed Python dependencies.

## Failure Modes
- Authenticated websocket success request omits `X-KAMN-Signer-Public-Key` and
  is rejected before upgrade.
- Invalid websocket version is not rejected with the deterministic reason code.
- Missing websocket upgrade headers are not rejected with the deterministic
  reason code.
- Missing auth headers are not rejected with unauthorized semantics.
- Local validation cannot execute the live Python probe when `cryptography` is
  unavailable; this must be reported as a local environment blocker, not hidden.

## Acceptance Criteria
- [ ] Red evidence captures CI failing with `expected websocket success status
      line`.
- [ ] Red local contract evidence captures the standalone websocket validator
      missing the signer public key header.
- [ ] The standalone websocket validator sends `X-KAMN-Signer-Public-Key` in
      authenticated websocket upgrade requests.
- [ ] Existing service API websocket live validation contract tests pass.
- [ ] Existing service API axum ingress websocket probe contract remains
      unchanged or equivalent.
- [ ] `cargo fmt --check`, strict workspace clippy, and `make check` remain
      green.

## Files To Touch
- `scripts/runtime/validate_service_api_websocket_live.sh`
- `scripts/runtime/test_validate_service_api_websocket_live.sh`
- A Rust or shell contract test only if needed to preserve fail-closed semantics
- This spec file

## Error Semantics
- Missing signer public key remains an auth failure.
- Invalid websocket version remains a 400 failure with
  `service_api_ws_version_header_invalid`.
- Missing websocket upgrade remains a 400 failure with
  `service_api_ws_upgrade_header_missing`.
- Missing auth remains a 401 failure with
  `service_api_auth_sender_did_header_missing`.

## Test Plan
- Red: record the Fast Gate CI failure with `expected websocket success status
  line`.
- Red: add a focused local contract assertion that the standalone websocket
  validator propagates `X-KAMN-Signer-Public-Key`; observe it fail before the
  implementation change.
- Green: update the standalone websocket validator to derive and send the signer
  public key header consistently with the axum ingress websocket probe.
- Integration: rerun the websocket live validation contract test and the axum
  ingress websocket-related contract tests.

## Completion Evidence
- Red CI: Fast Gate job `83784577752` failed on head
  `cc79c3d698e8b872eb3acaa7997a7d75ce240495` with `expected websocket success
  status line`.
- Local environment note: direct live validator execution failed before the
  websocket probe because local Python lacks `cryptography`.

## Shell-Surface Metrics
- `shell_loc_delta_estimate: +40`
- `rust_loc_delta_estimate: +120`
- `shell_to_rust_ratio_delta_estimate: -0.0001`
- `shell_surface_mitigation_issue: #7028`
