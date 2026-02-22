# Plan: #5678 Rebaseline R50 Spec-Volume Non-Regression Ratchet Markers

## Approach
1. Confirm current `specs/*` directory count and module count to quantify drift.
2. Add RED evidence by running the failing docs-contract test target.
3. Update R50 marker values and corresponding test constants.
4. Re-run the failing target and crate lint/format gates.

## Affected Modules
- `docs/review/gaps-and-issues-r50.md`
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`

## Risks and Mitigations
- Risk: accidental weakening of remediation constraints.
- Mitigation: only update non-regression ratchet values; keep remediation targets unchanged and preserve arithmetic assertions.
- Risk: repeated drift as new issue specs are added.
- Mitigation: set a bounded fresh ratchet that contains current state and update process comments to track future tranche reductions.

## Interfaces / Contracts
- Marker contracts remain same schema keys and same baseline=max pattern; only numeric values are rebaselined.

## ADR
- Not required; governance marker rebaseline only.
