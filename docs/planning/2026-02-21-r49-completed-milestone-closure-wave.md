# R49 Completed-Milestone Closure Hygiene Wave

## Summary
Closed completed open milestones that had `open_issues=0` to remove milestone-state drift and keep governance trackers aligned with delivered work.

## Eligibility Rule
- Eligible milestone: `open_issues=0`
- Ineligible milestone: `open_issues>0`

## Evidence Commands
Pre-closure open milestone inventory:

```bash
gh api repos/njfio/kamn/milestones?state=open --paginate --jq '.[] | [.number,.title,.open_issues,.closed_issues,.state] | @tsv'
```

Closure commands:

```bash
gh api -X PATCH repos/njfio/kamn/milestones/94 -f state=closed
gh api -X PATCH repos/njfio/kamn/milestones/95 -f state=closed
gh api -X PATCH repos/njfio/kamn/milestones/96 -f state=closed
```

Post-closure open milestone inventory:

```bash
gh api repos/njfio/kamn/milestones?state=open --paginate --jq '.[] | [.number,.title,.open_issues,.closed_issues,.state] | @tsv'
```

Closed milestone verification:

```bash
gh api repos/njfio/kamn/milestones?state=closed --paginate --jq '.[] | select(.number==94 or .number==95 or .number==96) | [.number,.title,.open_issues,.closed_issues,.state] | @tsv'
```

## Pre-Closure Snapshot
```text
94	R28.1 Cross-store replay production go/no-go integration	0	2	open
95	R48.1 Spec-volume and coherence batching mitigation	0	1	open
96	R49.1 Ignored-test periodic re-evaluation	0	1	open
97	R49.2 Completed-milestone closure hygiene wave	1	0	open
```

## Post-Closure Snapshot
```text
97	R49.2 Completed-milestone closure hygiene wave	1	0	open
```

## Closed Milestone Verification
```text
94	R28.1 Cross-store replay production go/no-go integration	0	2	closed
95	R48.1 Spec-volume and coherence batching mitigation	0	1	closed
96	R49.1 Ignored-test periodic re-evaluation	0	1	closed
```

## Deterministic Markers
- `completed_milestone_closure_wave_schema_version=kamn.review.completed-milestone-closure-wave.v1`
- `completed_milestone_closure_wave_target_open_issue_count=0`
- `completed_milestone_closure_wave_target_count=3`
- `completed_milestone_closure_wave_closed_milestone_ids_csv=94,95,96`
- `completed_milestone_closure_wave_closed_milestone_count=3`
- `completed_milestone_closure_wave_evidence_command_pre=gh api repos/njfio/kamn/milestones?state=open --paginate`
- `completed_milestone_closure_wave_evidence_command_post=gh api repos/njfio/kamn/milestones?state=open --paginate`
- `completed_milestone_closure_wave_post_open_milestone_count=1`

## Outcome
Milestones `#94`, `#95`, and `#96` are now closed with `open_issues=0` and removed from the open milestone inventory.
