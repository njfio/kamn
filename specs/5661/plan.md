# Plan: #5661 Verification Captured-At Format Contract

## Approach
1. Add RED conformance tests in `command_contract.rs` for malformed `_verification.captured_at` rejection.
2. Add RED unit coverage in `verify.rs` for deterministic captured-at format diagnostics.
3. Extend evidence verification validation to parse and enforce RFC3339 UTC-Z `captured_at` values.
4. Add R64 docs traceability markers and docs-contract tests.

## Affected Modules
- `crates/kamn-e2e-harness/src/verify.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `docs/research/*` and docs-contract tests for R64 traceability

## Risks and Mitigations
- Risk: brittle timestamp parsing producing false positives/negatives.
- Mitigation: explicit parser with focused invalid/valid conformance + unit tests, and positive-path regression validation.

## Interfaces / Contracts
- New deterministic error contract:
  - `evidence artifact invalid _verification.captured_at format: <artifact-path>`
- Existing contracts remain unchanged.

## ADR
- Not required. No architecture/dependency/protocol change.
