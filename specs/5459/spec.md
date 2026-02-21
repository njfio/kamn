# Issue #5459 Spec - Cross-store Replay Required Artifact Integration

- Status: Implemented
- Issue: #5459
- Parent: #3333
- Milestone: R28.1 Cross-store replay production go/no-go integration

## Problem Statement
`cross_store_replay_consistency` exists in production code and tests, but the release go/no-go gate required artifact inventory omits it. This allows release promotion to pass without explicitly asserting cross-store replay consistency signal in the required artifact contract.

## Scope
In scope:
- Add cross-store replay consistency to go/no-go required artifact IDs, manifest contract, and artifact lane registry.
- Ensure run-mode go/no-go lane can execute and validate cross-store replay artifact marker output.
- Update go/no-go lane tests and docs contract expectations impacted by required artifact expansion.

Out of scope:
- New storage backend implementation.
- Changes to cross-store replay divergence taxonomy semantics.

## Acceptance Criteria
- AC-1: go/no-go required artifact inventory includes `cross_store_replay_consistency` with deterministic expected lane and success marker contract.
- AC-2: go/no-go dry-run and run-mode contracts pass with updated required artifact count and status projection for cross-store replay artifact.
- AC-3: release manifest/checklist/docs references for required artifact inventory are updated and regression tests pass.
- AC-4: shell-surface DoD markers are reported with actual measured deltas.

## Conformance Cases
- C-01 (Functional, AC-1): manifest validation accepts `cross_store_replay_consistency` and fails when missing from required artifacts.
- C-02 (Functional, AC-2): run-mode executes cross-store replay lane command and validates expected marker.
- C-03 (Regression, AC-3): `scripts/runtime/test_run_go_no_go_gate_lane.sh` and checklist/docs tests pass with expanded artifact set.
- C-04 (Conformance, AC-4): PR/closure include `shell_loc_delta_actual`, `rust_loc_delta_actual`, `shell_to_rust_ratio_delta_actual`, and `shell_surface_ratio_target_status`.

## Success Metrics / Observable Signals
- `run_go_no_go_gate_lane` report contains cross-store replay status in artifact inventory with consistent GO/NO-GO behavior.
- Release manifest required artifact coverage includes cross-store replay consistency.
- CI fast gate remains green after contract updates.
