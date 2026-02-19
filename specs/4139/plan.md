# Issue #4139 Plan

- Issue: #4139
- Status: Reviewed

## Approach
1. Add a red metadata-contract assertion that expects explicit fuzz seed corpus drift markers.
2. Add parser failure-taxonomy assertions for both fuzz targets in metadata contract tests.
3. Extend invariant/fuzz strategy docs with explicit parser-taxonomy markers.
4. Extend docs contract tests to enforce those markers.
5. Run targeted tests and formatting checks.

## Risks and Mitigations
- Risk level: low
- Risks:
  - Metadata assertions may become brittle if corpus naming conventions change.
  - Taxonomy strings may drift without synchronized docs updates.
- Mitigations:
  - Keep expected marker set explicit and constrained to deterministic contract fields.
  - Pin docs markers with dedicated contract tests.

## Interface Contract
- Tests/docs/spec surface only.
- No production API behavior change.

## ADR
- Not required (contract/test governance scope).
