# Issue #5076 Spec

- Title: Task: integrate M4 escrow + M8 compliance contracts with legacy core types
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M4 and M8 currently model the same domains as `escrow.rs` and `content_lifecycle.rs`
with independent type systems and duplicated retention/transition semantics. This
creates drift risk and blocks clean runtime integration.

This task introduces explicit interop contracts so M4/M8 and legacy modules share
verified mapping behavior.

## Acceptance Criteria
- AC-1: M4 exposes deterministic interop mapping from legacy `EscrowStatus` to
  `DataLayerM4EscrowState` for representable states and fails closed for ambiguous
  settlement projections.
- AC-2: M8 exposes deterministic interop mapping between
  `DataLayerM8RetentionClass` and `ContentRetentionClass`, with fail-closed behavior
  for classes that cannot be represented in legacy lifecycle.
- AC-3: M8 exposes deterministic retention-window alignment signals against
  `content_lifecycle` profiles without altering M8 runtime retention policy.
- AC-4: Conformance tests cover bridge success paths and fail-closed error paths
  for both M4 and M8.
- AC-5: Shell/workflow/python/template LOC remains unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- `crates/kamn-core/src/data_layer_m4_escrow_integration.rs`
- `crates/kamn-core/src/data_layer_m8_compliance_lifecycle.rs`
- `crates/kamn-core/src/lib.rs` exports for new bridge contracts
- `crates/kamn-core/tests/data_layer_m4_escrow_integration.rs`
- `crates/kamn-core/tests/data_layer_m8_compliance_lifecycle.rs`
- `specs/5076/{spec.md,plan.md,tasks.md}`

Out of scope:
- Full module replacement/merge.
- M2/M3/M7/M9/M10 integration gaps.
- Dependency/protocol/wire-format changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | Map `EscrowStatus::{Funded,PartiallyReleased,Disputed,Released,Refunded}` to M4 state | Deterministic mapped state values |
| C-02 | AC-1 | Regression | Map `EscrowStatus::Resolved{released_total,refunded_total}` with mixed split | Fail-closed interop error |
| C-03 | AC-2 | Conformance | Map M8 retention classes to/from `ContentRetentionClass` | Deterministic mappings for representable classes |
| C-04 | AC-2 | Regression | Convert non-representable M8 classes (`LegalHold`,`Permanent`) to legacy class | Fail-closed interop error |
| C-05 | AC-3 | Functional | Evaluate M8 retention-window alignment signals against legacy profiles | `Extended` reports aligned; `Ephemeral`/`Standard` report drift; `LegalHold`/`Permanent` report no legacy counterpart |
| C-06 | AC-4 | Regression | Run M4+M8 scoped test suites | Bridge and existing behavior pass deterministically |
| C-07 | AC-5 | Regression | Inspect issue diff for shell/workflow/python/template files | Net shell-surface delta remains zero |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m4_escrow_integration`
- `cargo test -p kamn-core --test data_layer_m8_compliance_lifecycle`
- `cargo test -p kamn-core`

## Success Metrics
- Interop bridge contracts are public and tested.
- Critical M4/M8 semantic duplication is reduced to explicit, fail-closed mapping contracts.
- Shell-to-Rust posture is improved/neutral with zero shell delta.

## Verification Evidence
- RED: `cargo test -p kamn-core --test data_layer_m4_escrow_integration --test data_layer_m8_compliance_lifecycle`
  failed before bridge implementation with unresolved M4/M8 interop symbols and missing
  `TryFrom` contracts.
- GREEN: same scoped command passed after implementing bridge types/functions and exports.
- Regression:
  - `cargo fmt --check` passed.
  - `cargo clippy -p kamn-core -- -D warnings` passed.
  - `cargo test -p kamn-core` passed.

## Shell Surface Markers
- shell_loc_delta_actual: 0
- rust_loc_delta_actual: +199
- shell_to_rust_ratio_delta_actual: improved
- shell_surface_ratio_target_status: improved
