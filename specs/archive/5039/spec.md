# Issue #5039 Spec

- Title: Subtask: M10 partition lifecycle and archival recoverability contract suite
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Parent task `#5026` implemented baseline M10 partition lifecycle and archival
contracts, but recoverability validation remains shallow. M10 requires
deterministic recoverability evidence for archived/reattached partitions so
operator workflows can block unsafe restores and project stable readiness state.

PRD mapping:
- Section 13.2 archival + historical re-attachment lifecycle
- M10 scaling deliverables (archival pipeline and recoverability)
- M11/operator readiness dependencies on recoverability evidence

## Acceptance Criteria
- AC-1: Partition recoverability readiness is deterministically evaluated for
  archived and reattached partitions.
- AC-2: Recoverability readiness blocks active/non-historical partitions with
  stable ineligible-status reason markers.
- AC-3: Historical recovery readiness listing is deterministic and sorted for
  archived/reattached partitions only.
- AC-4: Unknown partition lookups fail closed with typed errors.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Extend existing `data_layer_m10_partition_archival` contracts with
  recoverability readiness projections.
- Add conformance tests for readiness outcomes and deterministic historical
  ordering.
- Export recoverability API through `kamn_core` root.

Out of scope:
- New shell/python/workflow orchestration.
- New dependencies or wire/protocol format changes.
- Live DB or object-store integration changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Evaluate readiness for archived partition | `Ready` decision with stable ready reason marker |
| C-02 | AC-1 | Conformance | Evaluate readiness after archived -> reattached transition | `Ready` decision remains stable for historical reattached partition |
| C-03 | AC-2 | Regression | Evaluate readiness for active partition | `Blocked` decision with stable ineligible-status reason marker |
| C-04 | AC-3 | Conformance | List historical recovery readiness for mixed partition states | Deterministic ordered list contains archived/reattached partitions only |
| C-05 | AC-4 | Regression | Evaluate readiness for unknown partition name | Fail-closed `PartitionNotFound` typed error |
| C-06 | AC-5 | Regression | Inspect diff paths and shell guardrails | No shell/workflow/python/template path changes; ratio/ceiling remain GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m10_partition_recoverability`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5039.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5039.json`

## AC Verification
| AC | ✅/❌ | Test(s) |
|---|---|---|
| AC-1 | ✅ | `spec_c01_archived_partition_recovery_readiness_is_ready`, `spec_c02_reattached_partition_recovery_readiness_remains_ready` |
| AC-2 | ✅ | `spec_c03_active_partition_is_blocked_for_recoverability` |
| AC-3 | ✅ | `spec_c04_historical_recovery_readiness_catalog_is_deterministic` |
| AC-4 | ✅ | `spec_c05_unknown_partition_lookup_fails_closed` |
| AC-5 | ✅ | `bash scripts/ci/check_shell_rust_ratio_guardrail.sh ...` and `bash scripts/ci/check_shell_loc_hard_ceiling.sh ...` with Rust-only diff |

## Success Metrics
- Recoverability APIs are exported and consumed from `kamn_core`.
- All ACs map to passing `spec_c0x_*` tests for recoverability suite.
- Shell-to-Rust ratio direction remains improved/neutral via Rust-only changes.
