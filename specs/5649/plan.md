# Plan: #5649 Chain Dump Genesis Anchor Verification

## Approach
1. Add RED conformance tests in `command_contract.rs` for genesis-anchor mismatch behavior.
2. Add RED unit test in `verify.rs` validating deterministic block-index `0` diagnostics.
3. Extend `verify_chain_dump` to enforce `GENESIS` anchor on the first block before pairwise continuity checks.
4. Add R60 traceability research doc and docs-contract test markers.

## Affected Modules
- `crates/kamn-e2e-harness/src/verify.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `docs/research/*` and docs-contract tests for R60 traceability

## Risks and Mitigations
- Risk: brittle marker parsing in string-based validation.
- Mitigation: maintain deterministic payload format assumptions and lock behavior with conformance/unit tests.

## Interfaces / Contracts
- New deterministic error contract:
  - `chain dump genesis anchor mismatch at block index 0`
- Existing contracts remain unchanged:
  - missing marker diagnostics
  - pairwise continuity mismatch diagnostics

## ADR
- Not required. No architecture/dependency/protocol change.
