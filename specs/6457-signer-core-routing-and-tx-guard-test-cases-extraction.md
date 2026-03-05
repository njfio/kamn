# Spec: Issue 6457 - Extract signer core-routing and tx-guard test cases

## Objective
Extract core-routing and transaction-guard integration scenarios from `crates/kamn-core/tests/signer_backend.rs` into a dedicated `signer_core_cases` module while preserving root entrypoint names and behavior.

## Inputs/Outputs
- Inputs:
  - Existing inline core-routing and tx-guard tests in `signer_backend.rs`.
- Outputs:
  - New `crates/kamn-core/tests/signer_backend/signer_core_cases.rs` containing extracted scenario bodies.
  - Root wrappers that keep original test names and delegate to `signer_core_cases`.
  - Updated `crates/kamn-core/tests/signer_backend_split_contract.rs` with core-case delegation and ownership markers.

## Boundaries/Non-goals
- No behavior changes to secure/local routing, provider-client mapping, missing-key handling, unsupported key reference handling, or tx-guard integration semantics.
- No production source changes under `crates/kamn-core/src`.
- No dependency additions.

## Failure modes
- Root file still owns inline core-routing/tx-guard test bodies.
- Split-contract guard fails to enforce core-case delegation markers.
- Extracted tests lose fail-closed assertions or transaction-guard integration coverage.

## Acceptance criteria (testable booleans)
- [ ] AC-1: `signer_core_cases.rs` exists and contains extracted core-routing/tx-guard scenario bodies.
- [ ] AC-2: root `signer_backend.rs` retains selected entrypoint names and delegates to `signer_core_cases` functions.
- [ ] AC-3: `signer_backend_split_contract.rs` enforces core-case delegation and ownership markers.
- [ ] AC-4: `cargo test -p kamn-core --test signer_backend_split_contract` passes.
- [ ] AC-5: `cargo test -p kamn-core --test signer_backend` passes.

## Files to touch
- `specs/6457-signer-core-routing-and-tx-guard-test-cases-extraction.md`
- `crates/kamn-core/tests/signer_backend.rs`
- `crates/kamn-core/tests/signer_backend/signer_core_cases.rs` (new)
- `crates/kamn-core/tests/signer_backend_split_contract.rs`

## Error semantics
- Preserve typed error assertion behavior for:
  - `MissingSigningKeyMaterial`
  - `UnsupportedKeyReference`
- Preserve assertion diagnostics for CI debuggability.

## Test plan
- Red:
  - Add core-case marker expectations to `signer_backend_split_contract.rs` before wiring module; verify failing split-contract test.
- Green:
  - Move selected core-routing/tx-guard test bodies to `signer_core_cases.rs` and delegate root wrappers.
- Refactor:
  - Deduplicate routing/request helper setup in the new module without behavior changes.
- Integration:
  - `cargo test -p kamn-core --test signer_backend_split_contract`
  - `cargo test -p kamn-core --test signer_backend`

## Phase 6 integration evidence
- Pending.

## Deviations
- None.
