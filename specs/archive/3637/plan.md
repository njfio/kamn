# Issue #3637 Plan

- Issue: `#3637`
- Status: `Completed`

## Approach
- Keep `signer_policy` as the policy boundary already extracted.
- Extract managed backend command/provenance functions into `signer/managed_backend.rs`.
- Extract nonce fetch/retry helpers into `signer/nonce.rs`.
- Re-export only the signer APIs required by `main.rs`/`main_tests` to preserve callsites.
- Preserve existing error strings/reason codes and test expectations.

## Affected Modules
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/signer/managed_backend.rs`
- `crates/kamn-node/src/signer/nonce.rs`
- `crates/kamn-node/src/main_tests/signer_tests.rs` (only if import/test paths need updates)
- `docs/foundation/runtime-network.md`

## Risks and Mitigations
- Risk: reason-code drift from moved error construction.
- Mitigation: preserve existing error literals and run signer regression suite.
- Risk: API visibility breakage for callsites in `main.rs` and tests.
- Mitigation: use explicit `pub(crate)` re-exports from `signer.rs` and compile/test gates.
- Risk: module extraction introduces semantic changes in retry pacing.
- Mitigation: keep deterministic backoff constants and unit tests unchanged.

## Interface Contract
- Public signer API consumed by runtime remains stable:
  - `resolve_kolme_live_managed_signer_required_marker`
  - `sign_kolme_live_managed_external_message`
  - `resolve_kolme_live_nonce`
- Managed backend and nonce internals become submodule-owned implementations.

## ADR
- No ADR required: decomposition keeps existing protocol and behavior contracts, and does not introduce new dependencies.
