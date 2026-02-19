# Issue #5103 Spec

- Title: Task: bridge M10 archival shred completeness to M8 compliance lifecycle
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M10 tracks partition-level `all_messages_shredded` as an independent boolean, while M8 is the authoritative source of message-level crypto-shred lifecycle state. Without a bridge, archival readiness can diverge from compliance truth.

## Acceptance Criteria
- AC-1: M10 exposes an additive bridge API that derives partition shred completeness from `DataLayerM8ComplianceRegistry` message lifecycle state.
- AC-2: Bridge deterministically updates partition `all_messages_shredded` from owner-scoped message IDs.
- AC-3: Missing/invalid compliance projection input fails closed with deterministic M10 reason taxonomy.
- AC-4: Existing M10 archival/recoverability behavior remains backward compatible.
- AC-5: Shell/workflow/python/template LOC remain unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- `crates/kamn-core/src/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/src/lib.rs`
- `specs/5103/{spec.md,plan.md,tasks.md}`

Out of scope:
- Replacing existing M10 APIs.
- Runtime/deployment changes.
- New dependencies.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Valid M10 partition + M8 owner/message set | M10 bridge returns derived projection report |
| C-02 | AC-2 | Conformance | Same partition projected before and after M8 crypto-shred transitions | Deterministic false->true completeness transition |
| C-03 | AC-3 | Regression | Projection includes message not present in M8 owner scope | Fail-closed M10 compliance projection error with stable reason code |
| C-04 | AC-4 | Regression | Existing M10 `spec_c01..spec_c05` and recoverability suite | Existing behavior remains green |
| C-05 | AC-5 | Regression | Shell guardrails | Zero shell delta; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m10_partition_archival`
- `cargo test -p kamn-core --test data_layer_m10_partition_recoverability`
- `cargo test -p kamn-core`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5103.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5103.json`

## Success Metrics
- M10 shred completeness can be derived from M8 lifecycle records via additive contract path.
- Compliance lookup failures are deterministic and fail closed.
- Shell governance metrics are unchanged or improved.
