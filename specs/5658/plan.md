# Plan: #5658 Verification Anchor Height Format Contract

## Approach
1. Add RED conformance tests in `command_contract.rs` for non-numeric block-height rejection.
2. Add RED unit coverage in `verify.rs` for deterministic block-height format diagnostics.
3. Extend evidence verification validation to parse and enforce numeric `block_height` values.
4. Add R63 docs traceability markers and docs-contract tests.

## Affected Modules
- `crates/kamn-e2e-harness/src/verify.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `docs/research/*` and docs-contract tests for R63 traceability

## Risks and Mitigations
- Risk: brittle parser logic around marker extraction.
- Mitigation: deterministic extraction from normalized anchor fragment and explicit conformance/unit tests for invalid and valid paths.

## Interfaces / Contracts
- New deterministic error contract:
  - `evidence artifact invalid _verification.kolme_anchor.block_height format: <artifact-path>`
- Existing contracts remain unchanged.

## ADR
- Not required. No architecture/dependency/protocol change.
