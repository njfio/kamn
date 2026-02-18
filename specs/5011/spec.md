# Issue #5011 Spec

- Title: Story: M8 compliance lifecycle with crypto-shredding and retention controls
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Deliver PRD M8 compliance contracts for deterministic retention evaluation,
legal-hold gating, and crypto-shredding controls. Story delivery is completed
through child task `#5024`.

## Acceptance Criteria
- AC-1: M8 retention/legal-hold/crypto-shred contracts are implemented with
  deterministic and fail-closed behavior.
- AC-2: Cross-owner compliance operations are denied with stable reason markers.
- AC-3: Story maps to PRD M8 requirements with reproducible conformance
  evidence and shell/workflow/python/template LOC unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Story-level completion evidence for child deliverable `#5024`.
- Story artifact normalization (`spec.md`, `plan.md`, `tasks.md`) to implemented lifecycle status.
- PRD M8 requirement mapping and deterministic evidence traceability.

Out of scope:
- New dependency/protocol/wire-format changes.
- Additional M8 expansion beyond accepted child task scope.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run `data_layer_m8_compliance_lifecycle` core lifecycle tests | Deterministic retention/legal-hold/crypto-shred behavior passes |
| C-02 | AC-2 | Conformance | Run owner-scope deny-path tests | Cross-owner operations fail closed with stable markers |
| C-03 | AC-3 | Regression | Story child-diff shell/rust audit + guardrail evidence | `shell_loc_delta_actual = 0`; ratio posture improved by rust-only changes |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m8_compliance_lifecycle`
- `cargo test -p kamn-core`
- Shell governance scripts are not required for child implementation because shell/workflow/python/template surfaces were unchanged.

## Success Metrics
- Story `#5011` closes with child task `#5024` merged and ACs mapped to passing deterministic tests.
- M8 compliance contract suite remains green in crate-level regression.
- Shell-to-Rust ratio posture is improved/neutral with zero shell delta.
