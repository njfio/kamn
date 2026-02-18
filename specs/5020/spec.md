# Issue #5020 Spec

- Title: Task: M4 integrate escrow state, scoped messaging, and settlement evidence storage
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
PRD M4 requires escrow-aware contracts that combine deterministic escrow lifecycle
state transitions, escrow-scoped message visibility, and immutable settlement evidence
storage. The current codebase has a generic escrow lifecycle (`escrow.rs`) and M2
gateway ABAC scaffolding, but no unified M4 data-layer contract that binds escrow
state, participant/auditor visibility gates, and settlement evidence integrity checks.

PRD mapping:
- Section 5.1.4 (escrows table, state, auditor threshold fields, settlement receipt hash)
- Section 9.2 (Escrow Participant Access, Escrow Auditor Access)
- Section 20 M4 milestone deliverables (state transitions, threshold key support, settlement receipts)
- Scenario 61 (threshold key reconstruction requirement)

## Acceptance Criteria
- AC-1: Escrow contract supports deterministic lifecycle transitions for
  `created -> funded -> active -> disputed -> released/refunded/expired` with fail-closed errors.
- AC-2: Escrow-scoped message visibility authorizes initiator/counterparty access and
  allows auditor access only when dispute is active and threshold-share reconstruction requirements are met.
- AC-3: Settlement evidence storage is append-only per escrow and persists deterministic
  settlement receipt hashes with integrity verification support.
- AC-4: M4 contracts expose stable reason-code/error markers for transition and authorization denials.
- AC-5: Shell/workflow/python LOC remains unchanged for this issue (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- New Rust M4 module in `kamn-core` for escrow state, scoped visibility, and settlement evidence contracts.
- Deterministic threshold-share validation model for auditor access gating.
- Conformance tests for transitions, visibility matrix, and settlement evidence integrity.
- Public API exports for follow-on M5+ integration.

Out of scope:
- Live PostgreSQL table wiring/migrations and gateway HTTP endpoints.
- External KMS/threshold cryptography libraries (contract-level verification only in this issue).
- Dependency additions or protocol/wire-format changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Apply valid escrow transition sequence | Lifecycle reaches expected terminal/active states with stable markers |
| C-02 | AC-1/AC-4 | Unit | Attempt invalid transitions (e.g., release before active) | Transition denied with typed fail-closed error |
| C-03 | AC-2 | Conformance | Visibility checks for initiator/counterparty/auditor/intruder | Participants allowed; auditor allowed only with dispute+threshold; intruder denied |
| C-04 | AC-3 | Conformance | Register settlement evidence for released/refunded escrow | Evidence accepted and digest/receipt marker deterministic |
| C-05 | AC-3/AC-4 | Regression | Tamper stored settlement evidence hash | Integrity verification fails with typed error |
| C-06 | AC-5 | Regression | Inspect issue diff for shell/workflow/python/template files | Net shell-surface delta remains zero |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m4_escrow_integration`
- `cargo test -p kamn-core spec_c0`
- `cargo test -p kamn-core`
- Shell governance scripts are not required because shell/workflow surfaces are unchanged.

## Success Metrics
- All ACs map to passing `spec_c0x_*` conformance tests.
- M4 contracts are exported via `kamn_core` for downstream integration lanes.
- Shell-to-Rust posture improves/neutral with zero shell LOC increase.

## Verification Evidence
- RED: `cargo test -p kamn-core --test data_layer_m4_escrow_integration` failed before implementation with unresolved `DataLayerM4*` symbols.
- GREEN: `cargo test -p kamn-core --test data_layer_m4_escrow_integration` passed after module implementation and exports.
- REGRESSION: `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`, and `cargo test -p kamn-core` pass.

## AC Verification
| AC | Result | Tests |
|---|---|---|
| AC-1 | ✅ | `spec_c01_escrow_state_machine_accepts_valid_transition_sequence`; `spec_c02_invalid_transition_paths_fail_closed` |
| AC-2 | ✅ | `spec_c03_scoped_message_visibility_enforces_participant_and_threshold_rules` |
| AC-3 | ✅ | `spec_c04_settlement_evidence_append_is_deterministic_for_final_states`; `spec_c05_settlement_evidence_hash_chain_detects_tamper` |
| AC-4 | ✅ | `spec_c02_invalid_transition_paths_fail_closed`; `spec_c05_settlement_evidence_hash_chain_detects_tamper` |
| AC-5 | ✅ | Diff inspection for issue files confirms Rust-only surface |

## Shell Surface Markers
- shell_loc_delta_actual: 0
- rust_loc_delta_actual: +1123
- shell_to_rust_ratio_delta_actual: -0.008229
- shell_surface_ratio_target_status: improved
