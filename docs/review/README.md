# Review Artifact Marker Contract

R43+ `gaps-and-issues` review artifacts must carry deterministic governance-vs-feature activity markers using key/value lines.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 43`:

- `governance_feature_activity_ratio_schema_version=kamn.review.governance-feature-activity-ratio.v1`
- `governance_activity_commit_count=<integer>`
- `feature_activity_commit_count=<integer>`
- `activity_total_commit_count=<integer>`
- `governance_activity_commit_ratio=<float>`
- `feature_activity_commit_ratio=<float>`

Contract invariants:

- `governance_activity_commit_count + feature_activity_commit_count = activity_total_commit_count`
- `governance_activity_commit_ratio ~= governance_activity_commit_count / activity_total_commit_count`
- `feature_activity_commit_ratio ~= feature_activity_commit_count / activity_total_commit_count`
- `governance_activity_commit_ratio + feature_activity_commit_ratio ~= 1.0`

This schema is enforced by `crates/kamn-core/tests/release_review_activity_ratio_docs_contract.rs`.
