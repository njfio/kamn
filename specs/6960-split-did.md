# 6960-split-did

## Objective
Reduce `crates/kamn-core/src/did.rs` to a thin root shell by extracting bounded submodules for DID document helpers, federated handshake/trust evaluation, and inline tests while preserving current behavior.

## Inputs/Outputs
- Inputs:
  - existing `crates/kamn-core/src/did.rs`
  - current `kamn-types` DID/value exports consumed by `kamn-core`
  - current `kamn-core` DID document and federated trust behavior
- Outputs:
  - thin root `crates/kamn-core/src/did.rs`
  - bounded submodules under `crates/kamn-core/src/did/`
  - hard-fail extraction contract in `crates/kamn-core/tests/`
  - updated spec evidence for the split

## Boundaries/Non-goals
- Do not redesign DID semantics.
- Do not change public DID formats.
- Do not change trust-policy outcomes beyond module movement.
- Do not widen the public API beyond what the root already exports.

## Failure modes
- Extraction contract does not fail when root shell is oversized or inline sections remain.
- Federated trust-store behavior changes after extraction.
- DID document canonicalization or verification-method validation changes after extraction.
- Test module wiring is lost and existing tests stop compiling.
- Touched-Rust size policy returns `NO-GO`.

## Acceptance criteria
- [ ] `crates/kamn-core/src/did.rs` is reduced to a thin root shell within the active file-size budget.
- [ ] DID concerns are split into bounded submodules with clear responsibilities.
- [ ] Existing federated handshake, DID document, and service-endpoint behavior remains unchanged under real tests.
- [ ] A hard-fail extraction contract covers the module boundary.
- [ ] Touched-Rust size policy returns `GO`.

## Files to touch
- `crates/kamn-core/src/did.rs`
- `crates/kamn-core/src/did/document.rs`
- `crates/kamn-core/src/did/federated.rs`
- `crates/kamn-core/src/did/tests.rs`
- optional second-level federated helpers under `crates/kamn-core/src/did/federated/`
- `crates/kamn-core/tests/did_module_extraction_contract.rs`
- `specs/6960-split-did.md`

## Error semantics
- Preserve existing typed errors and public return types.
- No silent fallback or error swallowing.
- Extraction contract must hard-fail with exact missing-marker / oversized-root conditions.

## Test plan
- Red extraction contract proving current `did.rs` is oversized and still contains inline sections.
- Green extraction contract after root reduction and module wiring.
- `cargo check -p kamn-core --lib`.
- Focused `kamn-core` test selection for DID behavior where current `main` permits it.
- Touched-Rust size policy run against the issue branch.
- If an unrelated current-main failure blocks a focused DID test target, record it explicitly in the spec as a deviation.
