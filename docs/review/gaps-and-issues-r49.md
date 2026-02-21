# KAMN Gaps and Issues Report

**As of:** R49 review artifact publication, issue `#5469` (2026-02-21)

---

## Baseline Snapshot

- Open issues: `1` (publication issue `#5469`)
- Open milestones: `1` (publication milestone `#98`)
- Remote branch heads: `50`
- Ignored-test drift checker: `pass` (`ignored_test_count=12`, `reason_codes=none`)
- Completed milestone closure wave: milestones `#94`, `#95`, `#96`, `#97` are closed

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

## Status Highlights

- R47 top-priority structural concern (`daemon_tests.rs` monolith) remains resolved.
- Branch hygiene remains controlled at `50` remote heads.
- R49 periodic ignored-test re-evaluation completed via `#5465` with baseline-aligned inventory.
- R49 completed-milestone closure hygiene wave completed via `#5467` with closures verified for `#94-#97`.

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
