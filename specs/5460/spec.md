# Issue #5460 Spec - Cross-store Go/No-go Lane Rust Harness Migration

- Status: Reviewed
- Issue: #5460
- Parent: #5459
- Milestone: R28.1 Cross-store replay production go/no-go integration

## Problem Statement
Issue `#5459` integrated cross-store replay consistency into go/no-go required artifacts but introduced a shell wrapper surface. We need to migrate that lane execution to a Rust harness path to reduce shell-surface ratio while preserving deterministic artifact markers and gate behavior.

## Scope
In scope:
- Replace shell wrapper lane execution for cross-store replay artifact with Rust binary/harness invocation.
- Keep go/no-go artifact marker contract stable (`cross_store_replay_consistency_policy_status=verified`).
- Remove superseded shell wrapper path and keep contract tests green.

Out of scope:
- Changing divergence taxonomy semantics.
- Expanding required artifact inventory beyond cross-store replay integration.

## Acceptance Criteria
- AC-1: Cross-store replay artifact command in go/no-go registry executes via Rust harness path (no shell wrapper dependency).
- AC-2: Go/no-go lane contract tests remain green with unchanged required marker semantics.
- AC-3: Shell-surface delta for this issue is net-negative and reported in PR/closure DoD markers.

## Conformance Cases
- C-01 (Functional, AC-1): go/no-go registry command for cross-store artifact uses `cargo run ... --bin ...` Rust harness invocation.
- C-02 (Regression, AC-2): `scripts/runtime/test_run_go_no_go_gate_lane.sh` passes with dry-run/run-mode marker checks.
- C-03 (Conformance, AC-3): PR includes measured shell/rust deltas and marks shell-surface status as improved/neutral.

## Success Metrics / Observable Signals
- `scripts/runtime/validate_cross_store_replay_consistency_contract_lane.sh` removed.
- go/no-go lane still emits deterministic cross-store status marker and passes CI.
