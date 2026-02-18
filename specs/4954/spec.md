# Issue #4954 Spec

- Title: Epic: enforce legacy-script deletion, spec archival, and hard shell LOC ceiling sustainment
- Status: Implemented
- Type: epic
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
The epic required a full governance correction tranche: explicit legacy-script deletion waves, completed-spec archival lifecycle controls, and hard shell budget/ratio CI gates.

## Acceptance Criteria
- AC-1: Full issue hierarchy exists and is delivered through closed child stories/tasks/subtasks.
- AC-2: Deletion-wave contracts identify and enforce superseded script removals with deterministic evidence.
- AC-3: Spec archival policy/tooling/wave lifecycle is implemented and contract-validated.
- AC-4: CI enforces hard shell LOC ceiling plus ratio/ratchet policy with deterministic outputs.
- AC-5: Governance/process artifacts remain synchronized with implemented controls.

## Scope
In scope:
- Story `#4955` (deletion waves), story `#4956` (archive lifecycle), story `#4957` (hard ceiling/ratio ratchet).

Out of scope:
- Unrelated feature expansion outside shell-surface sustainment.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | issue hierarchy closure state | all epic child stories/tasks/subtasks closed |
| C-02 | AC-2 | Integration/Regression | deletion manifest + stale-reference contract suites | deterministic wave enforcement |
| C-03 | AC-3 | Integration/Regression | archive policy/tool/wave suites | deterministic active/archive lifecycle enforcement |
| C-04 | AC-4 | Integration/Regression | ceiling/ratio/ratchet/wiring suites | CI merge-blocking policy coverage remains green |
| C-05 | AC-5 | Functional | updated epic/story/task lifecycle docs | governance artifacts synchronized |

## Test Mapping
- AC-2: `bash scripts/ci/test_check_superseded_script_deletion_manifest.sh`, `bash scripts/ci/test_check_stale_script_references.sh`
- AC-3: `bash scripts/ci/test_check_spec_archive_policy.sh`
- AC-4: `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh`, `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`, `bash scripts/ci/test_check_shell_surface_threshold_ratchet.sh`, `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
- AC-1/AC-5: closed hierarchy + synchronized lifecycle docs (`specs/4954/*`, `specs/4955/*`, `specs/4956/*`, `specs/4957/*`)

## Success Metrics
- Epic-level governance objectives are fully delivered and merge-protected.
