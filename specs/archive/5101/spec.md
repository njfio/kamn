# Issue #5101 Spec

- Title: Task: bridge M9 realtime delivery queue state to runtime backpressure contracts
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
`data_layer_m9_realtime_delivery.rs` models queue backpressure independently from the runtime backpressure controller. This creates two parallel overload taxonomies for the same queue state and leaves M9 dispatch orchestration without a typed bridge to runtime policy decisions.

## Acceptance Criteria
- AC-1: M9 exposes an additive projection contract that evaluates recipient queue state through runtime backpressure contracts.
- AC-2: Projection deterministically maps queue state to runtime decisions (`Accept`, `SlowProducer`, `RejectNewEnqueue`, `PurgeStalePeerQueue`) for fixed inputs.
- AC-3: Invalid backpressure projection inputs fail closed with deterministic M9 error taxonomy.
- AC-4: Existing M9 dispatch/presence/channel/anti-spam conformance behavior remains unchanged.
- AC-5: Shell/workflow/python/template LOC remain unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- `crates/kamn-core/src/data_layer_m9_realtime_delivery.rs`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs`
- `crates/kamn-core/src/lib.rs`
- `specs/5101/{spec.md,plan.md,tasks.md}`

Out of scope:
- Runtime transport behavior changes.
- Policy threshold redesign.
- Dependency additions.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Valid recipient queue projection request + runtime policy | M9 returns runtime decision contract with stable reason marker |
| C-02 | AC-2 | Conformance | Queue state slices across below-slow, slow, reject, disconnected stale | Deterministic action mapping for each state |
| C-03 | AC-3 | Regression | Invalid projection input (for example queue depth > capacity) | Fail-closed M9 projection error with stable reason marker |
| C-04 | AC-4 | Regression | Existing M9 `spec_c01..spec_c11` suite | Existing behavior remains green |
| C-05 | AC-5 | Regression | Shell guardrail checks | Zero shell delta; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery`
- `cargo test -p kamn-core`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5101.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5101.json`

## Success Metrics
- M9 queue state can be projected directly into runtime backpressure decisions with deterministic output.
- Bridge rejects invalid inputs fail closed with M9-specific error taxonomy.
- Shell governance posture is unchanged or improved.
