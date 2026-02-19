# Issue #5111 Spec

- Title: Task: execute implemented-spec archival wave and reduce active spec surface
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Implemented specs are accumulating under active `specs/<issue-id>/` directories despite a working archive policy/tooling pipeline. This increases navigation cost and documentation maintenance surface.

## Acceptance Criteria
- AC-1: Archive all currently eligible implemented issue specs in this wave.
- AC-2: Archive layout and pointer contracts remain valid (`specs/archive/<id>/`, `specs/<id>/ARCHIVED.md`, synchronized `specs/archive/index.md`).
- AC-3: `scripts/ci/check_spec_archive_policy.sh` passes after migration.
- AC-4: Shell/workflow/python/template LOC remain unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- `specs/archive/**`
- `specs/<id>/ARCHIVED.md` for archived issues
- `specs/5111/{spec.md,plan.md,tasks.md}`

Out of scope:
- Archive policy semantics changes
- New dependencies

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Eligible implemented issue-id set at wave start | All moved to `specs/archive/<id>/` with `ARCHIVED.md` pointers |
| C-02 | AC-2 | Conformance | Post-wave archive tree + index + pointers | Contract layout and index synchronization preserved |
| C-03 | AC-3 | Regression | `scripts/ci/check_spec_archive_policy.sh` | `status=ok` + `final_decision=GO` |
| C-04 | AC-4 | Regression | Shell guardrails | No shell LOC regression; ratio remains GO |

## Test Mapping
- `scripts/ci/archive_completed_specs.py --output-json /tmp/spec-archive-wave-5111-dryrun.json --issue-id ...`
- `scripts/ci/archive_completed_specs.py --apply --output-json /tmp/spec-archive-wave-5111-apply.json --issue-id ...`
- `scripts/ci/check_spec_archive_policy.sh --output-json /tmp/spec-archive-policy-5111.json`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/kamn-shell-loc-ceiling-5111.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/kamn-shell-rust-ratio-5111.json`

## Success Metrics
- Active implemented spec backlog reduced to zero for the selected eligibility set.
- Archive policy checks remain green.
- Shell governance posture unchanged or improved.
