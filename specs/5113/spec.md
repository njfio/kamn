# Issue #5113 Spec

- Title: Task: archive remaining implemented specs backlog wave
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Implemented specs remain in active `specs/<issue-id>/` directories after the first archive wave. The active spec surface is still larger than necessary and reduces navigability.

## Acceptance Criteria
- AC-1: Archive every currently eligible implemented active issue spec at wave start.
- AC-2: Preserve archive layout and pointer/index contracts (`specs/archive/<id>/`, `specs/<id>/ARCHIVED.md`, synced `specs/archive/index.md`).
- AC-3: `scripts/ci/check_spec_archive_policy.sh` passes after apply.
- AC-4: Shell governance remains green and neutral (`shell_loc_delta_actual = 0`, shell:Rust ratio GO).

## Scope
In scope:
- `specs/archive/**`
- `specs/<id>/ARCHIVED.md` for archived ids in this wave
- `specs/5113/{spec.md,plan.md,tasks.md}`

Out of scope:
- Archive policy semantic changes
- Runtime/product code changes
- New dependencies

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Implemented active issue-id set at wave start | All are migrated to `specs/archive/<id>/` with active pointer |
| C-02 | AC-2 | Conformance | Post-wave archive tree/index/pointers | Pointer + index contracts remain synchronized |
| C-03 | AC-3 | Regression | `scripts/ci/check_spec_archive_policy.sh` | `status=ok` and `final_decision=GO` |
| C-04 | AC-4 | Regression | Shell guardrail checkers | Ceiling + ratio remain `GO` with no shell LOC increase |

## Test Mapping
- `scripts/ci/archive_completed_specs.py --output-json /tmp/spec-archive-wave-5113-dryrun.json --issue-id ...`
- `scripts/ci/archive_completed_specs.py --apply --output-json /tmp/spec-archive-wave-5113-apply.json --issue-id ...`
- `scripts/ci/check_spec_archive_policy.sh --output-json /tmp/spec-archive-policy-5113.json`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/kamn-shell-loc-ceiling-5113.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/kamn-shell-rust-ratio-5113.json`

## Success Metrics
- Remaining implemented-active backlog at wave start was reduced to zero (`126` archived in this wave).
- Archive policy checks remain green.
- Shell governance metrics remain GO and neutral.
