# Objective
Split `crates/kamn-core/src/did_registry.rs` into bounded, concern-based modules while preserving DID registration, lifecycle mutation, persistence, finality tracking, and existing test behavior.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/src/did_registry.rs`
  - current DID registry tests and chain-adapter flows
  - existing persistence and finality helpers embedded in the root file
- Outputs:
  - a thin `did_registry.rs` root shell under the active file-size budget
  - bounded sibling modules for models, validation, store/persistence, chain submission, lifecycle mutation, and tests
  - extraction contract coverage enforcing the new module layout and active size limits

## Boundaries/Non-goals
- Do not change DID registry behavior or public API semantics.
- Do not change DID document formats, lifecycle reason codes, or chain submission outcomes.
- Do not redesign broader crate boundaries in this issue.
- Do not add dependencies.

## Failure modes
- Root file remains oversized after the split.
- Registration or lifecycle mutation semantics drift.
- Persistence/finality tracking behavior changes.
- Extracted files or functions still exceed active size limits.
- Error codes or fail-closed behavior change.

## Acceptance criteria
- [ ] The root file is reduced to a thin shell under the active file-size budget.
- [ ] Validation, persistence, chain-submission, lifecycle-mutation, and test seams are extracted into bounded modules.
- [ ] Existing DID registry tests remain green.
- [ ] No extracted file exceeds the active file-size limit.
- [ ] No extracted function exceeds the active function-size limit.

## Files to touch
- `specs/6854-split-did-registry.md`
- `crates/kamn-core/src/did_registry.rs`
- `crates/kamn-core/tests/did_registry_module_extraction_contract.rs`
- optional sibling modules under `crates/kamn-core/src/did_registry/`

## Error semantics
- Existing `DidRegistryError` behavior remains fail-closed.
- Registration, retry, rejection, finality, and lifecycle errors preserve their current codes/messages.
- No silent fallback or weakened validation may be introduced during the split.

## Test plan
1. Add a red extraction contract that fails while the root file remains oversized and the planned module layout is absent.
2. Re-run the existing DID registry test targets that cover registration, lifecycle mutation, persistence, and finality tracking.
3. Extract the file into bounded concern-based modules.
4. Re-run the extraction contract and real DID registry targets until green.
5. Run the touched-Rust size ratchet on the final write set.
