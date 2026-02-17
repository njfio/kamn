# Spec — Issue #4830

- Title: Subtask: retire static manifest maintenance path and add registry drift contract tests
- Parent: Parent task `#4816`
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Enforce registry-driven artifact maintenance by adding a fail-closed drift checker and deterministic tamper regression tests.

## Problem Statement

Without an explicit drift policy checker and tamper tests, static manifest/symlink maintenance can silently diverge from lane registry source-of-truth and regress generated architecture guarantees.

## Scope

In scope:
- add fail-closed drift policy checker script for lane registry artifacts
- add deterministic drift contract tests including tampered-manifest NO-GO path
- wire drift checks into framework regression entrypoint
- document retirement of manual static manifest maintenance path

Out of scope:
- redesigning lane registry schema introduced in `#4829`
- unrelated lane behavior or runtime semantics changes

## Acceptance Criteria

- AC-1: `check_lane_registry_drift.sh` emits deterministic GO/NO-GO decisions with stable reason taxonomy markers.
- AC-2: Drift contract tests cover pass path and tampered-manifest fail path.
- AC-3: Framework and CI regression suites include drift checks so static maintenance divergence fails closed.

## Conformance Cases

- C-01 (AC-1, Functional): `bash scripts/framework/check_lane_registry_drift.sh` over repository emits GO with `reason_codes=none`.
- C-02 (AC-2, Conformance): `bash scripts/framework/test_check_lane_registry_drift.sh` verifies pass markers and tampered-manifest NO-GO reason mapping.
- C-03 (AC-3, Integration/Regression): `bash scripts/framework/test_contract_framework.sh` and `bash scripts/ci/test_ci_tools.sh` pass with drift checker included.

## Success Metrics / Signals

- Static manifest maintenance path is retired in docs and replaced by registry drift checks.
- Drift checker failure modes emit deterministic reason taxonomy markers.
- CI regression remains green while enforcing drift guard.
