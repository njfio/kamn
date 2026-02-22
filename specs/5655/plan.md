# Plan: #5655 Verification Hash Format Contract

## Approach
1. Add RED conformance tests in `command_contract.rs` for invalid evidence-hash and anchor-tx-hash formats.
2. Add RED unit coverage in `verify.rs` for deterministic hash-format diagnostics.
3. Extend evidence verification-block validation to parse and enforce non-empty `sha256:` values.
4. Add R62 docs traceability artifacts and docs-contract checks.

## Affected Modules
- `crates/kamn-e2e-harness/src/verify.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `docs/research/*` and docs-contract tests for R62 traceability

## Risks and Mitigations
- Risk: brittle marker parsing due to text-based extraction.
- Mitigation: keep parser deterministic and constrained to required marker fragments; lock behavior with conformance and unit tests.

## Interfaces / Contracts
- New deterministic error contracts:
  - `evidence artifact invalid _verification.evidence_hash format: <artifact-path>`
  - `evidence artifact invalid _verification.kolme_anchor.tx_hash format: <artifact-path>`
- Existing contracts remain unchanged.

## ADR
- Not required. No architecture/dependency/protocol change.
