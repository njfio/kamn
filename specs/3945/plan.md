# Issue #3945 Plan

- Issue: #3945
- Status: Completed
- Spec: `specs/3945/spec.md`

## Implementation Approach
1. Add a new `kamn-node` contract test that validates required runtime test selectors remain present after extraction.
2. Encode deterministic reason taxonomy markers directly in the contract test and failure messages.
3. Add a docs section in `docs/ci/strategy.md` with explicit command-surface parity markers and guard command wiring.
4. Run targeted parity and docs-contract tests.

## Affected Modules
- `crates/kamn-node/tests/main_tests_command_surface_parity_contract.rs`
- `docs/ci/strategy.md`

## Risks and Mitigations
- Risk: false positives from text-based selector detection.
  - Mitigation: match concrete `fn <name>(` symbols in runtime test source files and keep required selector inventory small and explicit.
- Risk: docs marker drift.
  - Mitigation: add deterministic marker keys/values that can be asserted by docs-contract tests.

## Contracts and Interfaces
- Required selector paths remain in the form `main_tests::runtime_tests::<fn_name>`.
- Reason taxonomy version and code CSV are fixed in both docs and parity test.

## Verification Strategy
- RED: run new parity contract test before marker/docs additions (expected fail).
- GREEN: add selector parity checks + docs markers and rerun targeted tests.
- REGRESSION: run parity contract suite + `ci_strategy_docs`.
