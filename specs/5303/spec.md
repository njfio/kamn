# Issue #5303 Spec

- Title: Task: finalize R27.45 activation tracker statuses after #5301 merge
- Status: Accepted
- Priority: P2
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
After `#5301` merged, tracker documents still show it as in progress. This creates state drift between GitHub issue status and repo tracker docs.

## Scope
In:
- Update `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md` status markers for `#5301`.
- Update `docs/review/data-layer-roadmap.md` status markers for `#5301`.
- Keep milestone tracker list in sync with new closeout task.

Out:
- Runtime code changes.
- Shell/python/workflow changes.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 0
- shell_to_rust_ratio_delta_estimate: 0.0
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Activation plan marks `#5301` as completed/merged.
- AC-2: Data-layer roadmap marks `#5301` as completed/merged.
- AC-3: Milestone task listing includes this closeout task for traceability.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | activation plan content | `#5301` no longer appears as in-progress/current wave |
| C-02 | AC-2 | Functional | roadmap content | `#5301` no longer appears as current task |
| C-03 | AC-3 | Functional | milestone index | `#5303` listed in task hierarchy |

## Success Metrics
- No tracker entry reports `#5301` as in progress after merge.
- Milestone tracker and issue states remain synchronized.
