# Issue #5009 Spec

- Title: Story: M6 knowledge graph layer and trust propagation queries
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Deliver PRD M6 knowledge-graph contracts for owner-scoped node/edge
registration, deterministic trust-propagation queries, and portable graph export
projections. Story delivery is completed through child task `#5022`.

## Acceptance Criteria
- AC-1: Graph registry contracts support deterministic owner-scoped node/edge
  registration with fail-closed validation.
- AC-2: Trust-propagation queries compute bounded-depth owner-scoped scores
  with deterministic ordering and stable reason markers.
- AC-3: Portability/export projections remain deterministic and cross-owner graph
  access is denied fail closed.
- AC-4: Story maps to PRD M6 requirements with reproducible conformance evidence,
  and shell/workflow/python/template LOC remains unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Story-level completion evidence for child deliverable `#5022`.
- Story artifact normalization (`spec.md`, `plan.md`, `tasks.md`) to implemented
  lifecycle status.
- PRD M6 requirement mapping and deterministic evidence traceability.

Out of scope:
- New dependency/protocol/wire-format changes.
- Additional M6 expansion beyond the accepted child task scope.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run `data_layer_m6_graph_integration` registry tests | Owner-scoped node/edge registration contracts pass with deterministic validation behavior |
| C-02 | AC-2 | Conformance | Run trust-propagation scoring and ordering tests | Bounded-depth deterministic ranking and stable reason markers pass |
| C-03 | AC-3 | Conformance | Run portability/export and cross-owner deny-path tests | Deterministic projections pass and unauthorized access fails closed |
| C-04 | AC-4 | Regression | Story child-diff shell/rust audit + guardrail evidence | `shell_loc_delta_actual = 0`; ratio posture improved by rust-only changes |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m6_graph_integration`
- `cargo test -p kamn-core`
- Shell governance scripts are not required for child implementation because
  shell/workflow/python/template surfaces were unchanged.

## Success Metrics
- Story `#5009` closes with child task `#5022` merged and ACs mapped to passing
  deterministic tests.
- M6 graph integration contract suite remains green in crate-level regression.
- Shell-to-Rust ratio posture is improved/neutral with zero shell delta.
