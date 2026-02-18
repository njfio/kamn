# Spec - Issue #4163

- Title: Task: add multi-signer rotation preflight policy checks and deterministic evidence markers
- Parent: #4160
- Milestone: R27.20 Secret material zeroization and signer-rotation governance
- Status: Implemented
- Priority: P1

## Problem Statement

Multi-signer rotation readiness must enforce deterministic quorum and custody marker validation; otherwise tampered evidence or parity drift can pass preflight undetected.

## Objective

Close the parent task with AC/conformance mapping over merged subtask delivery:
- `#4169` failing-first quorum marker parity and tamper rejection tests,
- `#4170` deterministic custody reason-code mapping and docs-contract coverage.

## Scope

In scope:
- Rotation preflight marker completeness and quorum parity checks.
- Deterministic custody reason taxonomy outputs.
- Parent task lifecycle artifacts and verification mapping.

Out of scope:
- External approval workflow automation.
- Enterprise custody platform integration.

## Acceptance Criteria

- AC-1: Rotation preflight checks enforce required markers and quorum policy.
- AC-2: Drift/mismatch emits deterministic fail-closed reason markers.
- AC-3: Failing-to-passing tests validate quorum/custody behavior.
- AC-4: Unit/Functional/Integration/Regression coverage remains present and passing.

## Conformance Cases

- C-01 (AC-1): `bash scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh` passes with quorum parity and marker checks.
- C-02 (AC-2): same checker test passes with deterministic custody reason taxonomy output mapping.
- C-03 (AC-3): `bash scripts/kolme/test_run_local_kolme_live_deployment_preflight_lane.sh` passes and keeps rotation preflight checks fail-closed.
- C-04 (AC-4): `cargo test -p kamn-core --test kolme_devnet_ops_docs` and `cargo test -p kamn-core --test release_gonogo_checklist_docs` pass.

## Success Metrics

- Rotation preflight marker and custody/quorum parity drift is rejected with stable deterministic reasons.
- Docs-contract suites enforce rotation readiness marker governance in CI.
