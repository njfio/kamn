# Issue #3891 Tasks

- Issue: #3891
- Status: Completed

## Ordered Tasks
- T1 (Red): extended `scripts/runtime/test_run_go_no_go_gate_lane.sh` with failing checks for readiness-marker omission and fail-closed budget behavior.
- T2 (Green): updated `scripts/runtime/go_no_go_gate_lane_contract.py` to:
  - enforce readiness marker completeness in policy evaluation
  - convert `runtime_budget_exceeded` to fail-closed policy behavior
  - add `readiness_marker_missing` fault profile for deterministic validation
- T3 (Refactor): centralized readiness marker projection values before policy evaluation and reused them for report/stdout markers.
- T4 (Regression): added readiness marker fault-profile assertions and runtime-budget fail-closed assertions in script harness (stdout + JSON payload).
- T5 (Docs): updated `docs/ci/strategy.md` go/no-go marker contract section for readiness marker requirements and budget fail-closed semantics.
- T6 (Verify): ran:
  - `bash scripts/runtime/test_run_go_no_go_gate_lane.sh`
  - `cargo test -p kamn-core --test ci_strategy_docs`

## Completion Evidence
- Go/no-go policy now fails closed for missing readiness markers and runtime budget overflow with deterministic reason codes, and docs-contract coverage remains green.
