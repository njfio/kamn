# Issue #5007 Spec

- Title: Story: M4 escrow-scoped messaging and settlement evidence integration
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Deliver PRD M4 escrow integration contracts for deterministic lifecycle
transitions, escrow-scoped visibility rules, and append-only settlement evidence
integrity verification. Story delivery is completed through child task `#5020`.

## Acceptance Criteria
- AC-1: M4 escrow state machine transitions are deterministic and fail closed for
  invalid transition attempts.
- AC-2: Escrow-scoped visibility authorization allows participant and threshold-
  eligible auditor paths and denies unauthorized access with stable markers.
- AC-3: Settlement evidence contracts are append-only, deterministic, and detect
  hash-chain tamper conditions.
- AC-4: Story maps to PRD M4 requirements with reproducible conformance evidence,
  and shell/workflow/python/template LOC remains unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Story-level completion evidence for child deliverable `#5020`.
- Story artifact normalization (`spec.md`, `plan.md`, `tasks.md`) to implemented
  lifecycle status.
- PRD M4 requirement mapping and deterministic evidence traceability.

Out of scope:
- New dependency/protocol/wire-format changes.
- Additional M4 expansion beyond the accepted child task scope.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run `data_layer_m4_escrow_integration` transition tests | Valid transitions succeed; invalid transitions fail closed with typed markers |
| C-02 | AC-2 | Conformance | Run escrow visibility matrix tests | Participant/auditor authorize paths pass only under required conditions; unauthorized cases deny deterministically |
| C-03 | AC-3 | Conformance | Run settlement evidence append/tamper tests | Append-only evidence checks pass and tamper detection fails deterministically |
| C-04 | AC-4 | Regression | Story child-diff shell/rust audit + guardrail evidence | `shell_loc_delta_actual = 0`; ratio posture improved by rust-only changes |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m4_escrow_integration`
- `cargo test -p kamn-core`
- Shell governance scripts are not required for child implementation because
  shell/workflow/python/template surfaces were unchanged.

## Success Metrics
- Story `#5007` closes with child task `#5020` merged and ACs mapped to passing
  deterministic tests.
- M4 escrow integration contract suite remains green in crate-level regression.
- Shell-to-Rust ratio posture is improved/neutral with zero shell delta.
