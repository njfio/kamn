# Issue #5026 Spec

- Title: Task: M10 deliver scaling controls, partition lifecycle, and archival export path
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
PRD M10 requires deterministic controls for monthly partition lifecycle
management, archival export indexing, and historical partition re-attachment.
Current codebase contains runtime/network scaling primitives but no dedicated
M10 contract surface for partition naming rules, retention-window archival
eligibility, and archival-index consistency markers.

PRD mapping:
- Section 5.4 (monthly partitioning and naming convention)
- Section 13.2 (monthly partition archival to object storage + archival index)
- Section 13.2 re-attachment workflow for historical queries
- Milestone table M10 deliverables (partition management + archival pipeline)

## Acceptance Criteria
- AC-1: Partition lifecycle contract produces deterministic monthly partition
  identifiers (`messages_YYYY_MM`) and planning outputs for future partitions.
- AC-2: Archival eligibility contract deterministically selects partitions older
  than the active retention window and only archives shred-complete partitions.
- AC-3: Archival index contract records deterministic object-storage export
  metadata and supports partition re-attachment state transitions.
- AC-4: Invalid month identifiers and illegal lifecycle transitions fail closed
  with stable error markers.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- New Rust M10 module in `kamn-core` for monthly partition lifecycle planning,
  archival eligibility, and archival-index/reattach transitions.
- Conformance tests for partition naming, archival candidate selection, and
  archival index integrity markers.
- Public API exports for downstream M11 and operator-runbook integration lanes.

Out of scope:
- Live database DDL/pg_partman orchestration and object-storage network I/O.
- New shell/python/workflow/template orchestration.
- New dependencies or wire/protocol format changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Plan future partitions from a reference month | Deterministic `messages_YYYY_MM` sequence for configured horizon |
| C-02 | AC-2 | Conformance | Evaluate archival eligibility with retention window + shred-complete markers | Only eligible partitions become archival candidates in stable order |
| C-03 | AC-3 | Conformance | Archive eligible partition and inspect archival index projection | Deterministic object-storage URI + format/checksum markers are recorded |
| C-04 | AC-3/AC-4 | Regression | Re-attach archived partition and inspect lifecycle transition | Transition succeeds only from archived state and becomes queryable |
| C-05 | AC-4 | Regression | Use invalid month identifiers or duplicate registrations | Fail-closed typed errors with stable reason markers |
| C-06 | AC-5 | Regression | Inspect issue diff paths | No shell/python/workflow/template path changes |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m10_partition_archival`
- `cargo test -p kamn-core spec_c0`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5026.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5026.json`

## AC Verification
| AC | ✅/❌ | Test(s) |
|---|---|---|
| AC-1 | ✅ | `spec_c01_partition_naming_and_future_planning_are_deterministic` |
| AC-2 | ✅ | `spec_c02_archival_due_selection_respects_retention_window_and_shred_completeness` |
| AC-3 | ✅ | `spec_c03_archival_index_records_and_reattach_transition_are_deterministic` |
| AC-4 | ✅ | `spec_c04_invalid_month_identifiers_and_illegal_transitions_fail_closed` |
| AC-5 | ✅ | `bash scripts/ci/check_shell_rust_ratio_guardrail.sh ...` and `bash scripts/ci/check_shell_loc_hard_ceiling.sh ...` with Rust-only diff |

## Success Metrics
- M10 contracts are exported via `kamn_core` for downstream integration lanes.
- All ACs map to passing `spec_c0x_*` conformance tests.
- Shell-to-Rust ratio direction remains improved/neutral through Rust-only changes.
