# Plan: #5652 Verification Anchor Finality Value Contract

## Approach
1. Add RED conformance tests in `command_contract.rs` for non-`FINAL` finality rejection.
2. Add RED unit coverage in `verify.rs` for deterministic invalid-finality diagnostics.
3. Extend evidence verification marker validation to enforce exact finality value `FINAL`.
4. Add R61 docs traceability artifacts and docs-contract tests.

## Affected Modules
- `crates/kamn-e2e-harness/src/verify.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `docs/research/*` and docs-contract tests for R61 traceability

## Risks and Mitigations
- Risk: brittle text-marker extraction around finality parsing.
- Mitigation: deterministic parser helper scoped to required marker and conformance tests covering invalid/valid value paths.

## Interfaces / Contracts
- New deterministic error contract:
  - `evidence artifact invalid _verification.kolme_anchor.finality value`
- Existing marker-presence contracts remain unchanged.

## ADR
- Not required. No architecture/dependency/protocol change.
