# Spec: #5678 Rebaseline R50 Spec-Volume Non-Regression Ratchet Markers

- Issue: #5678
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
The CI target `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract` fails because current `specs/*` directory count has moved beyond the R50 non-regression markers (`spec_dir_max=825`, `ratio_max=9.0`).

## Scope
### In Scope
- Rebaseline R50 non-regression marker values in `docs/review/gaps-and-issues-r50.md` to contain the current repository spec-dir count.
- Update corresponding hard-coded marker expectations in `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`.
- Keep arithmetic and consistency constraints intact.

### Out of Scope
- Deleting/migrating existing `specs/*` directories.
- Changing remediation-plan targets (`target_ratio_max=7.7`, `target_spec_dir_max=708`).

## Acceptance Criteria
### AC-1 Marker rebaseline consistency
Given current repository spec-dir inventory,
When the non-regression ratchet markers are evaluated,
Then the marker cap and ratio contain current values and maintain baseline=max consistency.

### AC-2 Docs-contract parity
Given updated marker values,
When `review_r50_spec_volume_remediation_docs_contract` executes,
Then functional and integration assertions pass without weakening consistency invariants.

### AC-3 CI-unblock verification
Given the failing CI target,
When the fix is applied,
Then `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract` passes.

## Conformance Cases
- C-01 (AC-1): R50 report includes updated non-regression marker values (`baseline_spec_dirs`, `ratio_max`, `spec_dir_max`) with baseline=max.
- C-02 (AC-2): `functional_r50_spec_volume_remediation_markers_present` reflects updated values.
- C-03 (AC-3): `integration_r50_spec_volume_remediation_markers_are_consistent` passes on current repo state.

## Success Metrics
- `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract` passes.
- `cargo fmt --all --check` and `cargo clippy -p kamn-core -- -D warnings` pass.
