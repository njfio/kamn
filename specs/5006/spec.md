# Issue #5006 Spec

- Title: Story: M3 metadata and blind-index search surfaces
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Deliver PRD M3 searchable-encryption contracts for deterministic blind-index
normalization, owner-scoped exact-match lookup, and metadata filter surfaces.
Story delivery is completed through child task `#5019`.

## Acceptance Criteria
- AC-1: Blind-index computation normalizes values deterministically and produces
  owner-scoped exact-match tokens.
- AC-2: Blind-index search contracts are fail-closed for unsupported/invalid
  inputs and enforce owner scoping.
- AC-3: Metadata filtering contracts support deterministic sender/recipient/session/escrow/type/time predicates with stable ordering.
- AC-4: Story maps to PRD M3 requirements with reproducible conformance evidence,
  and shell/workflow/python/template LOC remains unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Story-level completion evidence for child deliverable `#5019`.
- Story artifact normalization (`spec.md`, `plan.md`, `tasks.md`) to implemented
  lifecycle status.
- PRD M3 requirement mapping and deterministic evidence traceability.

Out of scope:
- New dependency/protocol/wire-format changes.
- Additional M3 expansion beyond the accepted child task scope.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run `data_layer_m3_blind_index_search` normalization and owner-scope tests | Equivalent semantic inputs normalize to identical indexes; distinct owner salts produce distinct tokens |
| C-02 | AC-2 | Conformance | Run blind-index query and invalid-input tests | Exact matches only; unsupported modes and empty inputs fail closed |
| C-03 | AC-3 | Conformance | Run metadata query filter and ordering tests | Deterministic ordered result sets honoring all supplied filters |
| C-04 | AC-4 | Regression | Story child-diff shell/rust audit + guardrail evidence | `shell_loc_delta_actual = 0`; ratio posture improved by rust-only changes |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m3_blind_index_search`
- `cargo test -p kamn-core`
- Shell governance scripts are not required for child implementation because
  shell/workflow/python/template surfaces were unchanged.

## Success Metrics
- Story `#5006` closes with child task `#5019` merged and ACs mapped to passing
  deterministic tests.
- M3 blind-index and metadata contract suite remains green in crate-level regression.
- Shell-to-Rust ratio posture is improved/neutral with zero shell delta.
