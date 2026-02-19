# Issue #3794 Spec

- Title: Subtask: implement transport resilience local-heavy lane artifact and policy checker
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
The transport resilience local-heavy lane already validates retry/reconnect behavior, but it lacks an explicit deterministic retry/reconnect marker contract status field that can be consumed by downstream policy and docs drift contracts.

## Acceptance Criteria
- AC-1: Local-heavy transport resilience run-lane summary emits a deterministic retry/reconnect marker contract status field.
- AC-2: Local-heavy transport resilience policy checker fails closed when retry/reconnect marker contract status is missing or mismatched.
- AC-3: Contract-lane aggregation and docs references include the deterministic retry/reconnect marker contract status.
- AC-4: Unit/Functional/Integration/Regression evidence for this marker contract is present and passing.

## Scope
In scope:
- `scripts/runtime/live_transport_fault_matrix_live_contract.py`
- `scripts/runtime/validate_live_transport_fault_matrix_live_contract_lane.sh`
- `scripts/runtime/test_validate_live_transport_fault_matrix_live.sh`
- `scripts/runtime/test_check_live_transport_fault_matrix_live_policy.sh`
- `scripts/runtime/test_validate_live_transport_fault_matrix_live_contract_lane.sh`
- `docs/planning/kolme-devnet-ops.md`
- `specs/3794/{spec.md,plan.md,tasks.md}`

Out of scope:
- CI-fast exclusion policy wiring for this lane (tracked in `#3795`)
- New transport protocol behavior
- New dependencies

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | dry-run transport resilience lane execution | output includes `retry_reconnect_marker_contract_status=verified` and report JSON field |
| C-02 | AC-2 | Regression | policy checker against lane report | policy output/report include deterministic retry/reconnect marker contract status; mismatch fails closed |
| C-03 | AC-3 | Integration | contract-lane runner output aggregation | lane output/report include propagated retry/reconnect marker contract status from policy surface |
| C-04 | AC-3 | Regression | docs contract checks | Kolme devnet ops docs include retry/reconnect marker contract status declaration for the transport resilience lane |
| C-05 | AC-4 | Regression | shell guardrails | no shell/rust ratio or hard-ceiling regression introduced by this change |

## Test Mapping
- `bash scripts/runtime/test_validate_live_transport_fault_matrix_live.sh`
- `bash scripts/runtime/test_check_live_transport_fault_matrix_live_policy.sh`
- `bash scripts/runtime/test_validate_live_transport_fault_matrix_live_contract_lane.sh`
- `cargo test -p kamn-node --test kolme_devnet_ops_docs`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3794.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3794.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3794.json`

## Success Metrics
- Retry/reconnect marker contract status is deterministic and visible in lane, policy, and contract-lane outputs.
- Policy and docs checks fail closed on drift.
- Shell-surface guardrails remain green.
