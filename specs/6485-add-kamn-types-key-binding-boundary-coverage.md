## Objective

Add explicit boundary coverage for `kamn-types` DID/public-key binding helpers and shared re-export
types so the crate’s public identity surface is verified beyond basic canonical parsing.

## Inputs/Outputs

- Inputs:
  - `kamn_types::AgentDid::with_public_key_hex_binding(method_specific_id, public_key_hex)`
  - `kamn_types::did::AgentDid::with_public_key_hex_binding(method_specific_id, public_key_hex)`
  - `AgentDid::key_binding_fingerprint()`
  - `AgentDid::ensure_public_key_hex_binding(public_key_hex)`
  - `parse_agent_did_canonical(value)`
  - `parse_kamn_did_canonical(value)`
  - `DidDocument`, `DidService`, `DidVerificationMethod`, `AgentDidMetadata`
- Outputs:
  - deterministic equivalent key-binding DID generation across top-level and module imports
  - deterministic error propagation for invalid public-key hex and canonical parse failures
  - explicit construction coverage for shared DID boundary types exposed by `kamn-types`

## Boundaries/Non-goals

- No production API changes
- No workflow, CI, or dependency changes
- No modifications to DID parsing or key-binding algorithms
- No new docs-only markers in this slice

## Failure modes

- Top-level and `kamn_types::did` imports drift and produce different key-binding results
- Generated agent DIDs stop exposing or validating key-binding fingerprints correctly
- Invalid public-key hex stops surfacing as `AgentDidKeyBindingError::InvalidPublicKeyHex`
- Canonical parse helpers stop preserving underlying `MissingMethodSpecificId` and
  `InvalidShape` failures
- Re-exported DID boundary types stop being constructible through `kamn-types`

## Acceptance criteria

- [ ] A test proves top-level and `kamn_types::did` key-binding generation produce equivalent DIDs
- [ ] A test proves generated agent DIDs expose a key-binding fingerprint and validate against the original public key hex
- [ ] A test proves invalid public-key hex returns `AgentDidKeyBindingError::InvalidPublicKeyHex`
- [ ] A test proves `parse_agent_did_canonical(...)` preserves `MissingMethodSpecificId`
- [ ] A test proves `parse_kamn_did_canonical(...)` preserves `InvalidShape`
- [ ] A test proves `DidDocument`, `DidService`, `DidVerificationMethod`, and `AgentDidMetadata` are constructible from `kamn-types` imports
- [ ] `cargo test -p kamn-types -- --nocapture` passes

## Files to touch

- `specs/6485-add-kamn-types-key-binding-boundary-coverage.md`
- `crates/kamn-types/tests/key_binding_boundary_contract.rs`
- `crates/kamn-types/tests/key_binding_boundary_integration.rs`
- `fixtures/ci/test_file_size_policy_baseline.env` (only if new test targets change inventory)

## Error semantics

- `AgentDid::with_public_key_hex_binding(...)` and `ensure_public_key_hex_binding(...)` continue to
  return `AgentDidKeyBindingError` variants unchanged
- Canonical parse helpers continue to wrap underlying parser failures in `SharedDidParseError`
- No new error variants are introduced

## Test plan

- Add a contract test that requires a dedicated key-binding boundary integration target
- Add integration tests for equivalent key-binding generation across import paths, fingerprint
  exposure and validation, invalid public-key hex failure, canonical error propagation, and shared
  type construction through `kamn-types`
- Run:
  - `cargo test -p kamn-types --test key_binding_boundary_contract -- --nocapture`
  - `cargo test -p kamn-types --test key_binding_boundary_integration -- --nocapture`
  - `cargo test -p kamn-types -- --nocapture`
  - `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` if test inventory changes

## Deviations

- None
