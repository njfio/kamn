# Spec: Issue #4432

Status: Implemented
Issue: #4432
Parent: #4429
Milestone: R27.38 SDK-client readiness, deployment packaging, and live-validation governance
Priority: P1

## Problem Statement

SDK websocket subscription and Python packaging checks exist, but deterministic subscription and
publish-readiness taxonomy/evidence marker surfaces are underspecified. This creates drift risk for
release readiness and CI evidence governance.

## Scope

In scope:
- RED tests for websocket subscription drift and packaging metadata regressions.
- Deterministic taxonomy marker outputs for websocket subscription and Python packaging readiness.
- Deterministic evidence marker outputs in SDK live-validation scripts.
- SDK docs updates for subscription/packaging governance marker surfaces.

Out of scope:
- New transport protocols or advanced client orchestration APIs.

## Acceptance Criteria

AC-1:
Given SDK contract scripts, when websocket subscription taxonomy markers drift, then tests fail
closed with deterministic diagnostics.

AC-2:
Given Python packaging contract scripts, when publish-readiness taxonomy markers drift, then tests
fail closed with deterministic diagnostics.

AC-3:
Given subscription + packaging live validations, when reports are emitted, then deterministic
taxonomy version/reason-code CSV/evidence markers are present in stdout and JSON.

AC-4:
Given SDK docs checks, when subscription/packaging marker surfaces drift, then checks fail closed.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
  - Expectation: websocket subscription taxonomy markers are required and deterministic.

- C-02 (AC-2, Functional/Regression):
  - Tests:
    - `bash scripts/sdk/test_run_python_sdk_packaging_contract.sh`
    - `bash scripts/sdk/test_validate_python_sdk_packaging_live.sh`
  - Expectation: publish-readiness taxonomy markers are required and deterministic.

- C-03 (AC-3, Integration):
  - Tests:
    - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
    - `bash scripts/sdk/test_validate_python_sdk_packaging_live.sh`
  - Expectation: deterministic taxonomy/evidence markers in live validation outputs.

- C-04 (AC-4, Docs):
  - Tests:
    - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
    - `bash scripts/sdk/test_run_python_sdk_packaging_contract.sh`
  - Expectation: docs include subscription + packaging taxonomy/evidence marker surfaces.

## Success Metrics / Observable Signals

- Subscription and publish-readiness drift is fail-closed and deterministic.
- SDK live evidence emits stable machine-readable taxonomy marker surfaces.
