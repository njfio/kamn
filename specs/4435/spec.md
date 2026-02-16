# Spec: Issue #4435

Status: Implemented
Issue: #4435
Parent: #4431
Milestone: R27.38 SDK-client readiness, deployment packaging, and live-validation governance
Priority: P1

## Problem Statement

SDK HTTP request-error contract tests do not yet fail RED on missing deterministic taxonomy marker
surfaces. Without explicit RED checks, request/error drift can be accepted silently.

## Scope

In scope:
- RED assertions for request-error taxonomy/evidence marker drift in SDK contract tests.
- RED assertions for docs marker drift in SDK HTTP helper docs.

Out of scope:
- Runtime implementation changes beyond test wiring.

## Acceptance Criteria

AC-1:
Given SDK service-client contract tests, when taxonomy markers are missing, then tests fail
deterministically.

AC-2:
Given SDK live-validation tests, when taxonomy/evidence markers are missing, then tests fail
deterministically.

AC-3:
Given docs marker checks, when expected taxonomy markers are missing from SDK docs, then tests fail
closed.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
  - Expectation: fails RED when taxonomy markers are absent.

- C-02 (AC-2, Functional/Regression):
  - Test: `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
  - Expectation: fails RED when live taxonomy/evidence markers are absent.

- C-03 (AC-3, Docs):
  - Tests:
    - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
    - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
  - Expectation: docs drift is fail-closed.

## Success Metrics / Observable Signals

- RED tests fail before implementation and deterministically explain drift.
