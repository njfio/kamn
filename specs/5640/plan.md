# Plan: #5640 Evidence `_verification` Block Enforcement

## Approach
1. Add RED tests in `command_contract.rs` for missing `_verification` and missing required nested markers.
2. Implement deterministic evidence JSON scanner in verify flow:
   - recurse evidence dir
   - select `.json` artifacts excluding manifest/support files
   - enforce `_verification` required markers
   - stable path-sorted validation order
3. Re-run command contract and full crate verification gates.
4. Add R57 docs/research marker artifact and docs contract test.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs` (verify command path hook)
- `crates/kamn-e2e-harness/src/verify.rs` (artifact marker validation)
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/*docs_contract.rs` (new R57 docs contract)
- `docs/research/`

## Risks and Mitigations
- Risk: over-broad file selection introduces false failures.
  - Mitigation: deterministic include/exclude rules with explicit support-file exclusions.
- Risk: nondeterministic filesystem ordering.
  - Mitigation: sort candidate paths before validation.
- Risk: regressions in existing verify success path.
  - Mitigation: retain existing success tests and full crate gates.

## Interfaces/Contracts
- Verify report JSON shape remains unchanged.
- Error messages include missing marker path and artifact path.
