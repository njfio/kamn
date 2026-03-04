# Spec: Issue #6319 - enforce no-manual HMAC/HKDF helper regression contract

## Objective

Add a workspace-facing contract test that fail-closes if manual HMAC/HKDF helper function
signatures are reintroduced in production crypto modules, while also asserting the RustCrypto
backend markers remain present.

## Inputs/Outputs

- Inputs:
  - existing crypto module sources:
    - `crates/kamn-crypto/src/direct_message_crypto.rs`
    - `crates/kamn-core/src/group_channel_crypto.rs`
  - current local module-level assertions for manual helper removal and backend markers.
- Outputs:
  - a dedicated kamn-core contract test lane source that:
    - verifies forbidden helper signatures are absent in targeted production modules.
    - verifies required backend markers are present in targeted production modules.
  - lane passes under `cargo test -p kamn-core --test issue_6319_hmac_hkdf_regression_contract`.

## Boundaries/Non-goals

- In scope:
  - add one focused Rust contract test file under `crates/kamn-core/tests`.
  - enforce known forbidden manual helper signatures and required backend markers.
- Out of scope:
  - any crypto algorithm or wire-format changes.
  - any CI workflow topology changes.
  - broad static-analysis framework work beyond this targeted regression contract.

## Failure modes

- FM-1: manual helper signature `fn hmac_sha256(` reappears in production crypto modules.
- FM-2: manual helper signature `fn hkdf_sha256_derive_32(` reappears in production crypto modules.
- FM-3: required backend markers `rustcrypto.hkdf.sha256.v1` or `rustcrypto.hmac.sha256.v1`
  disappear from active production crypto modules.
- FM-4: contract test drifts to non-production files and misses the actual guarded modules.

## Acceptance criteria (testable booleans)

- AC-1: contract fails when forbidden helper signatures appear in guarded production crypto modules.
- AC-2: contract fails when required backend markers are absent from guarded production crypto modules.
- AC-3: contract passes on current mainline module sources with no behavior changes.
- AC-4: `cargo test -p kamn-core --test issue_6319_hmac_hkdf_regression_contract` passes.

## Files to touch

- `specs/6319-no-manual-hmac-hkdf-regression-contract.md`
- `crates/kamn-core/tests/issue_6319_hmac_hkdf_regression_contract.rs`

## Error semantics

- Contract lane fails closed with explicit assertion messages that identify:
  - offending module path,
  - missing marker or forbidden signature.
- No runtime production error behavior changes are introduced.

## Test plan

- RED:
  - add test assertions for marker/signature contracts and run lane against intentionally stale
    assumptions to confirm failure.
- GREEN:
  - align assertions with current production module structure and marker strings.
- REFACTOR:
  - deduplicate source-check helper logic in the test file for clarity and maintainability.
- INTEGRATION:
  - run dedicated lane plus existing adjacent crypto lane:
    - `cargo test -p kamn-core --test issue_6319_hmac_hkdf_regression_contract`
    - `cargo test -p kamn-core --test issue_5921_production_message_crypto`

## Phase 6 integration evidence

- `cargo test -p kamn-core --test issue_6319_hmac_hkdf_regression_contract`:
  - pass (`2 passed, 0 failed`)
- `cargo test -p kamn-core --test issue_5921_production_message_crypto`:
  - pass (`17 passed, 0 failed`)

## Deviations

- None.
