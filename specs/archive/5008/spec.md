# Issue #5008 Spec

- Title: Story: M5 vector embedding layer and semantic search
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Deliver PRD M5 vector intelligence contracts for owner-scoped embedding
registration, deterministic semantic ranking, and anomaly-scoring evaluation.
Story delivery is completed through child task `#5021`.

## Acceptance Criteria
- AC-1: Embedding registry contracts accept owner-scoped inserts and preserve
  deterministic append-only integrity behavior.
- AC-2: Semantic query contracts return deterministic owner-scoped ranking with
  fail-closed handling for invalid prerequisites.
- AC-3: Anomaly scoring contracts compute deterministic centroid-distance
  thresholds with stable reason markers.
- AC-4: Story maps to PRD M5 requirements with reproducible conformance evidence,
  and shell/workflow/python/template LOC remains unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Story-level completion evidence for child deliverable `#5021`.
- Story artifact normalization (`spec.md`, `plan.md`, `tasks.md`) to implemented
  lifecycle status.
- PRD M5 requirement mapping and deterministic evidence traceability.

Out of scope:
- New dependency/protocol/wire-format changes.
- Additional M5 expansion beyond the accepted child task scope.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run `data_layer_m5_vector_integration` embedding registry tests | Owner-scoped inserts and append-only integrity behaviors pass deterministically |
| C-02 | AC-2 | Conformance | Run semantic query and privacy-mode tests | Deterministic top-k ranking and fail-closed invalid-prerequisite paths pass |
| C-03 | AC-3 | Conformance | Run anomaly-scoring threshold tests | Stable anomaly decisions and reason markers match deterministic expectations |
| C-04 | AC-4 | Regression | Story child-diff shell/rust audit + guardrail evidence | `shell_loc_delta_actual = 0`; ratio posture improved by rust-only changes |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m5_vector_integration`
- `cargo test -p kamn-core`
- Shell governance scripts are not required for child implementation because
  shell/workflow/python/template surfaces were unchanged.

## Success Metrics
- Story `#5008` closes with child task `#5021` merged and ACs mapped to passing
  deterministic tests.
- M5 vector integration contract suite remains green in crate-level regression.
- Shell-to-Rust ratio posture is improved/neutral with zero shell delta.
