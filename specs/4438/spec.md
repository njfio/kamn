# Spec: Issue #4438

Status: Implemented
Issue: #4438
Parent: #4432
Milestone: R27.38 SDK-client readiness, deployment packaging, and live-validation governance
Priority: P1

## Problem Statement

SDK subscription and packaging checks need deterministic taxonomy and publish-readiness evidence
outputs so release decisions remain stable and auditable.

## Scope

In scope:
- Deterministic websocket subscription taxonomy marker outputs.
- Deterministic packaging publish-readiness taxonomy marker outputs.
- Deterministic live evidence markers for subscription + packaging contract status.
- Docs updates for subscription/packaging marker surfaces.

Out of scope:
- New transport protocol implementations.

## Acceptance Criteria

AC-1:
Given Rust SDK subscription contract runner output, when successful, then deterministic subscription
taxonomy version/reason CSV markers are present in stdout and JSON.

AC-2:
Given Python packaging contract/live outputs, when successful, then deterministic publish-readiness
taxonomy version/reason CSV markers are present in stdout and JSON.

AC-3:
Given live validation outputs, when successful, then deterministic combined subscription/packaging
evidence markers are present.

AC-4:
Given docs checks, when marker surfaces drift, then checks fail closed.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
  - Expectation: deterministic subscription taxonomy markers emitted.

- C-02 (AC-2, Functional/Integration):
  - Tests:
    - `bash scripts/sdk/test_run_python_sdk_packaging_contract.sh`
    - `bash scripts/sdk/test_validate_python_sdk_packaging_live.sh`
  - Expectation: deterministic publish-readiness taxonomy markers emitted.

- C-03 (AC-3, Integration):
  - Tests:
    - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
    - `bash scripts/sdk/test_validate_python_sdk_packaging_live.sh`
  - Expectation: deterministic live evidence contract markers emitted.

- C-04 (AC-4, Docs):
  - Tests:
    - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
    - `bash scripts/sdk/test_run_python_sdk_packaging_contract.sh`
  - Expectation: docs marker surface remains pinned.

## Success Metrics / Observable Signals

- Stable subscription and packaging publish-readiness taxonomy markers across runs.
