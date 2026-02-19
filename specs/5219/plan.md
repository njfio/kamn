# Issue #5219 Plan

- Issue: #5219
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Approach
1. Add a new docs-contract test that scans `docs/review/gaps-and-issues-r*.md` and enforces marker keys for releases >= R43.
2. Run the new test before adding markers to capture RED failure evidence.
3. Add marker block to `docs/review/gaps-and-issues-r43.md` and schema guidance in `docs/review/README.md`.
4. Re-run targeted tests and clippy for GREEN evidence.

## Risks and Mitigations
- Risk: brittle parsing across review-doc formatting changes.
  - Mitigation: use explicit `key=value` marker lines independent of prose formatting.
- Risk: pre-R43 docs fail contract retroactively.
  - Mitigation: enforce markers only for R43+ files by filename release number.

## Interfaces / Contracts
- Marker schema keys (R43+):
  - `governance_feature_activity_ratio_schema_version`
  - `governance_activity_commit_count`
  - `feature_activity_commit_count`
  - `activity_total_commit_count`
  - `governance_activity_commit_ratio`
  - `feature_activity_commit_ratio`
