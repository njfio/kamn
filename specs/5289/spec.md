# Issue #5289 Spec

- Title: Task: add Phase-6 retention+archival execution tick orchestration contracts
- Status: Reviewed
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Phase-6 has retention-to-archival gating (`#5285`) and archival retry-policy projection (`#5287`) but still lacks a single deterministic orchestration boundary that executes retention due lookup, crypto-shredding, partition shred projection, and archival selection in one tick.

## Scope
In:
- Add one deterministic Phase-6 orchestration function composing M8 + M10 execution flow.
- Add stable reason markers and deterministic execution report fields.
- Add conformance coverage for happy-path orchestration and fail-closed legal-hold / projection errors.
- Update data-layer tracker docs to advance current wave.

Out:
- Live scheduler daemon loops in `kamn-node`.
- External storage I/O adapters.
- Shell/python/workflow/template changes.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 320
- shell_to_rust_ratio_delta_estimate: -0.0004
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: A Phase-6 orchestration tick deterministically composes M8 retention due lookup, crypto-shredding execution, M10 shred-completeness projection, and archival planning.
- AC-2: Orchestration output report includes stable reason markers and deterministic ordering for shredded message IDs, projection reports, and archival entries.
- AC-3: Owner-scope/legal-hold/projection-input failures are explicit and fail closed.
- AC-4: Unit/Functional/Integration/Regression coverage and required validation commands pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | owner with retention-due messages and eligible partition | shredded messages + projected completeness + archived entries in one deterministic tick |
| C-02 | AC-1 | Integration | composed M8+M10 registries with two partitions | one tick report includes per-partition projection and archival counts |
| C-03 | AC-2 | Unit | unsorted partition-message map + due set | sorted deterministic ordering in report artifacts |
| C-04 | AC-2 | Unit | no due messages | zero-count deterministic report with success reason marker |
| C-05 | AC-3 | Regression | legal-hold message in due path | fail-closed orchestration error with stable reason marker |
| C-06 | AC-3 | Regression | empty partition projection message list for mapped month | fail-closed orchestration error with stable reason marker |
| C-07 | AC-4 | Verification | fmt/clippy + targeted orchestration tests | all checks pass |

## Success Metrics
- Story `#5253` advances from isolated contracts to a deterministic Phase-6 execution tick contract.
- No shell-surface growth while extending compliance automation behavior.
