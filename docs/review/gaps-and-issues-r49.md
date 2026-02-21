# KAMN Gaps and Issues Report

**As of:** R49 review artifact publication, issue `#5469` (2026-02-21)
**Post-publication revalidation:** issue `#5485` (2026-02-21)
**Post-publication branch-count reconciliation:** issue `#5495` (2026-02-21)

---

## Baseline Snapshot

- Open issues: `1` (publication issue `#5469`)
- Open milestones: `1` (publication milestone `#98`)
- Remote branch heads: `50`
- Ignored-test drift checker: `pass` (`ignored_test_count=12`, `reason_codes=none`)
- Completed milestone closure wave: milestones `#94`, `#95`, `#96`, `#97` are closed

Publication snapshot values above remain historical to R49.3 artifact publication time.

## Deterministic Baseline Evidence Commands

```bash
gh issue list --state open --limit 200 --json number,title,milestone --jq '.[] | [.number,.title, (.milestone.title // "none")] | @tsv'
gh api repos/njfio/kamn/milestones?state=open --paginate --jq '.[] | [.number,.title,.open_issues,.closed_issues,.state] | @tsv'
git ls-remote --heads origin | wc -l
bash scripts/ci/check_ignored_test_inventory_drift.sh --baseline-file fixtures/ci/ignored_test_inventory_baseline.json --metadata-file fixtures/ci/ignored_test_inventory_metadata.json --promotion-criteria-file fixtures/ci/ignored_test_promotion_criteria.json --output-json /tmp/ignored-test-inventory-drift-report-r49-review.json
gh api repos/njfio/kamn/milestones?state=closed --paginate --jq '.[] | select(.number==94 or .number==95 or .number==96 or .number==97) | [.number,.title,.open_issues,.closed_issues,.state] | @tsv'
```

## Captured Outputs

Open issues:

```text
5469	Task: publish R49 gaps-and-issues review artifact with refreshed baseline markers	R49.3 Review artifact publication and baseline refresh
```

Open milestones:

```text
98	R49.3 Review artifact publication and baseline refresh	1	0	open
```

Closed milestone verification:

```text
94	R28.1 Cross-store replay production go/no-go integration	0	2	closed
95	R48.1 Spec-volume and coherence batching mitigation	0	1	closed
96	R49.1 Ignored-test periodic re-evaluation	0	1	closed
97	R49.2 Completed-milestone closure hygiene wave	0	1	closed
```

Ignored-test drift checker summary:

```text
status=pass
ignored_test_count=12
reason_codes=none
```

Post-publication branch-count reconciliation provenance:

```text
issue=5495
scope=branch_remote_head_count_marker_update_to_50
```

## Post-Publication Revalidation Snapshot (R50.8)

- Open issues: `0`
- Open milestones: `0`
- Remote branch heads: `50`
- Ignored-test drift checker: `pass` (`ignored_test_count=12`, `reason_codes=none`)

## Deterministic Revalidation Evidence Commands

```bash
gh issue list --repo njfio/kamn --state open --limit 200
gh api repos/njfio/kamn/milestones?state=open --paginate --jq 'length'
git ls-remote --heads origin | wc -l
bash scripts/ci/check_ignored_test_inventory_drift.sh --baseline-file fixtures/ci/ignored_test_inventory_baseline.json --metadata-file fixtures/ci/ignored_test_inventory_metadata.json --promotion-criteria-file fixtures/ci/ignored_test_promotion_criteria.json --output-json /tmp/ignored-test-inventory-drift-report-r49-revalidate.json
```

## Revalidation Captured Outputs

Open issues:

```text
(none)
```

Open milestones count:

```text
0
```

Remote branch heads:

```text
50
```

Ignored-test drift checker summary:

```text
status=pass
ignored_test_count=12
reason_codes=none
```

## Status Highlights

- R47 top-priority structural concern (`daemon_tests.rs` monolith) remains resolved.
- Branch hygiene remains controlled at `50` remote heads (post-publication revalidation snapshot).
- R49 periodic ignored-test re-evaluation completed via `#5465` with baseline-aligned inventory.
- R49 completed-milestone closure hygiene wave completed via `#5467` with closures verified for `#94-#97`.
- Post-publication production feature delivery is reconciled via issue `#5499` and PR `#5500`.

## Governance-Feature Activity Ratio Markers (R49)

- governance_feature_activity_ratio_schema_version=kamn.review.governance-feature-activity-ratio.v1
- governance_activity_commit_count=3
- feature_activity_commit_count=0
- activity_total_commit_count=3
- governance_activity_commit_ratio=1.0000
- feature_activity_commit_ratio=0.0000

## Deterministic Markers

- `r49_review_artifact_schema_version=kamn.review.gaps-and-issues-r49.v1`
- `r49_review_baseline_capture_date=2026-02-21`
- `r49_review_baseline_branch_remote_head_count=50`
- `r49_review_baseline_open_issue_count=1`
- `r49_review_baseline_open_milestone_count=1`
- `r49_review_ignored_test_inventory_count=12`
- `r49_review_ignored_test_periodic_review_status=completed`
- `r49_review_ignored_test_periodic_review_issue=5465`
- `r49_review_milestone_closure_wave_closed_ids_csv=94,95,96,97`
- `r49_review_milestone_closure_wave_issue=5467`
- `r49_review_post_publication_revalidation_date=2026-02-21`
- `r49_review_post_publication_issue=5485`
- `r49_review_post_publication_feature_issue=5499`
- `r49_review_post_publication_feature_pr=5500`
- `r49_review_post_publication_branch_count_reconciliation_issue=5495`
- `r49_review_post_publication_branch_remote_head_count=50`
- `r49_review_post_publication_open_issue_count=0`
- `r49_review_post_publication_open_milestone_count=0`
- `r49_review_post_publication_ignored_test_inventory_count=12`
