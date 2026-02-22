# Plan: #5646 Chain Dump Hash Continuity Verification

## Approach
1. Add RED conformance tests in `command_contract.rs` for missing block hash markers and continuity mismatch.
2. Extend verify-path chain dump validation with deterministic block hash continuity checks.
3. Keep validation dependency-free by parsing deterministic chain markers from chain dump text.
4. Add R59 docs/research markers and docs contract tests for traceability.

## Affected Modules
- `crates/kamn-e2e-harness/src/verify.rs`
- `crates/kamn-e2e-harness/src/lib.rs` (verify path wiring unchanged except behavior enforcement)
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `docs/research/*` and docs contract tests for R59 traceability

## Risks and Mitigations
- Risk: brittle parsing if marker-order assumptions drift.
- Mitigation: enforce deterministic marker format in conformance tests; fail with explicit diagnostics.

## Interfaces / Contracts
- Verify command must fail with deterministic errors:
  - `chain dump block missing block_hash marker`
  - `chain dump block missing previous_block_hash marker`
  - `chain dump hash continuity mismatch at block index <n>`

## ADR
- Not required. No architecture/dependency/protocol change.
