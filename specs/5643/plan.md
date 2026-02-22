# Plan: #5643 Chain Dump Marker Validation

## Approach
1. Add RED tests in `command_contract.rs` for missing chain dump markers.
2. Implement deterministic chain dump content validation in verify flow.
3. Re-run command contract suite and full crate gates.
4. Add R58 docs/research marker artifact and docs contract test.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/src/verify.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/*docs_contract.rs` (new R58 docs contract)
- `docs/research/`

## Risks and Mitigations
- Risk: over-strict marker checks on minimal chain dumps.
  - Mitigation: require only minimal deterministic markers listed in scope.
- Risk: regressions in existing verify tests.
  - Mitigation: preserve success-path report assertions and run full crate tests.

## Interfaces/Contracts
- Verify report JSON shape unchanged.
- Deterministic missing-marker error strings for chain dump content checks.
