# Spec: Issue #4431

Status: Implemented
Issue: #4431
Parent: #4429
Milestone: R27.38 SDK-client readiness, deployment packaging, and live-validation governance
Priority: P1

## Problem Statement

Rust/Python SDK HTTP helper coverage exists but does not expose a fully pinned deterministic
request-error taxonomy/evidence marker surface in the SDK contract lane outputs. Without this
surface, request/error drift classification can become unstable and harder to audit in release
promotion.

## Scope

In scope:
- RED tests for SDK HTTP request-error drift and unstable classification.
- Deterministic taxonomy/evidence markers in Rust SDK HTTP contract runner/live validation outputs.
- Deterministic normalization checks for Python SDK legacy adapter error classification.
- Docs updates for SDK HTTP helper taxonomy and evidence markers.

Out of scope:
- New network protocols or advanced client orchestration features.
- Marketplace-level SDK package distribution automation.

## Acceptance Criteria

AC-1:
Given SDK HTTP contract tests, when taxonomy/evidence markers drift or are missing, then tests fail
closed with deterministic messages.

AC-2:
Given Rust SDK service-client contract runner/live validation execution, when outputs are produced,
then deterministic request-error taxonomy version/reason-code CSV/evidence markers are emitted in
stdout and JSON.

AC-3:
Given Python SDK legacy backend error envelopes, when reasons are normalized, then reason-code
classification remains deterministic and regression-pinned.

AC-4:
Given SDK documentation checks, when marker surfaces drift, then docs-contract checks fail closed.

## Conformance Cases

- C-01 (AC-1, Functional/Conformance):
  - Tests:
    - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
    - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
  - Expectation: missing taxonomy/evidence marker surfaces fail RED before implementation.

- C-02 (AC-2, Integration):
  - Tests:
    - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
    - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
  - Expectation: deterministic taxonomy and evidence markers pass after implementation.

- C-03 (AC-3, Unit/Regression):
  - Test: `python3 -m unittest tests.python.test_sdk`
  - Expectation: legacy backend adapter reason normalization is deterministic and regression-pinned.

- C-04 (AC-4, Docs):
  - Tests:
    - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
    - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
  - Expectation: docs reference deterministic taxonomy/evidence marker surfaces.

## Success Metrics / Observable Signals

- SDK HTTP request/error drift tests fail closed with deterministic diagnostics.
- Rust SDK lane outputs include machine-readable taxonomy markers.
- Python legacy adapter reason normalization remains deterministic across runs.
