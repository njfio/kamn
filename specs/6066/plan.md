# Plan: Issue #6066

## Approach
1. Add RED contract test in `crates/kamn-node/src/main_tests/signer_tests.rs` (or dedicated test file) asserting signer adapter struct derive does not include `Clone`.
2. Remove `Clone` derive from `crates/kamn-node/src/signer/signer_adapter.rs`.
3. Run targeted signer tests to verify no call sites depend on cloning.
4. Validate with fmt + clippy + targeted tests.

## Affected Modules
- `crates/kamn-node/src/signer/signer_adapter.rs`
- `crates/kamn-node/src/main_tests/signer_tests.rs` (or equivalent contract test location)
- `specs/6066/spec.md`
- `specs/6066/plan.md`
- `specs/6066/tasks.md`

## Risks / Mitigations
- Risk: hidden clone usage in signer call paths.
  Mitigation: targeted compile + signer test pass before PR.

## Interfaces / Contracts
- Signer adapter ownership contract becomes non-cloneable by type system.
