# Spec: Issue #4437

Status: Reviewed
Issue: #4437
Parent: #4432
Milestone: R27.38 SDK-client readiness, deployment packaging, and live-validation governance
Priority: P1

## Problem Statement

SDK websocket subscription and packaging tests do not currently enforce deterministic taxonomy marker
surfaces for drift regressions. RED checks are needed so drift is fail-closed before promotion.

## Scope

In scope:
- RED assertions for websocket subscription taxonomy marker drift.
- RED assertions for packaging publish-readiness taxonomy marker drift.
- RED assertions for docs marker drift for SDK subscription/packaging surfaces.

Out of scope:
- Implementation of marker emission (handled by #4438).

## Acceptance Criteria

AC-1:
Given Rust SDK contract tests, when websocket taxonomy markers are absent, then tests fail
deterministically.

AC-2:
Given Python packaging contract/live tests, when publish-readiness taxonomy markers are absent, then
tests fail deterministically.

AC-3:
Given SDK docs checks, when required marker surfaces are absent, then tests fail closed.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
  - Expectation: fails RED on missing websocket taxonomy markers.

- C-02 (AC-2, Functional/Regression):
  - Tests:
    - `bash scripts/sdk/test_run_python_sdk_packaging_contract.sh`
    - `bash scripts/sdk/test_validate_python_sdk_packaging_live.sh`
  - Expectation: fails RED on missing packaging taxonomy markers.

- C-03 (AC-3, Docs):
  - Tests:
    - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
    - `bash scripts/sdk/test_run_python_sdk_packaging_contract.sh`
  - Expectation: docs marker drift is fail-closed.

## Success Metrics / Observable Signals

- RED checks fail deterministically before implementation.
