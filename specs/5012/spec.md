# Issue #5012 Spec

- Title: Story: M9 real-time delivery, presence, and flow control
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Deliver PRD M9 realtime delivery contracts for deterministic dispatch ACKs,
presence visibility controls, and backpressure escalation markers. Story
delivery is completed through child task `#5025`.

## Acceptance Criteria
- AC-1: M9 dispatch/presence/backpressure contracts are implemented with
  deterministic and fail-closed behavior.
- AC-2: Cross-owner operations are denied with stable reason markers.
- AC-3: Story maps to PRD M9 requirements with reproducible conformance
  evidence and shell/workflow/python/template LOC unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Story-level completion evidence for child deliverable `#5025`.
- Story artifact normalization (`spec.md`, `plan.md`, `tasks.md`) to implemented lifecycle status.
- PRD M9 requirement mapping and deterministic evidence traceability.

Out of scope:
- New dependency/protocol/wire-format changes.
- Additional M9 expansion beyond accepted child task scope.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run `data_layer_m9_realtime_delivery` core ACK/presence tests | Deterministic dispatch/presence behavior passes |
| C-02 | AC-2 | Conformance | Run cross-owner deny-path tests | Owner-scope violations fail closed with stable markers |
| C-03 | AC-3 | Regression | Story child-diff shell/rust audit + guardrail evidence | `shell_loc_delta_actual = 0`; ratio posture improved by rust-only changes |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery`
- `cargo test -p kamn-core`
- Shell governance scripts are not required for child implementation because shell/workflow/python/template surfaces were unchanged.

## Success Metrics
- Story `#5012` closes with child task `#5025` merged and ACs mapped to passing deterministic tests.
- M9 realtime contract suite remains green in crate-level regression.
- Shell-to-Rust ratio posture is improved/neutral with zero shell delta.
