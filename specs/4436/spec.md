# Spec: Issue #4436

Status: Reviewed
Issue: #4436
Parent: #4431
Milestone: R27.38 SDK-client readiness, deployment packaging, and live-validation governance
Priority: P1

## Problem Statement

SDK HTTP contract lanes need explicit deterministic error-taxonomy and normalized evidence output
surfaces so request/error outcomes remain stable for release governance.

## Scope

In scope:
- Deterministic request-error taxonomy marker emission in Rust SDK HTTP contract runner.
- Deterministic taxonomy/evidence marker emission in Rust SDK live validation.
- Python regression coverage for deterministic legacy adapter reason normalization.
- SDK docs updates for taxonomy/evidence references.

Out of scope:
- Non-core SDK transport features.

## Acceptance Criteria

AC-1:
Given Rust SDK HTTP contract runner success, when output/report is produced, then taxonomy version,
reason CSV, and taxonomy status markers are present and deterministic.

AC-2:
Given Rust SDK live-validation success, when output/report is produced, then deterministic HTTP
taxonomy contract markers and reason taxonomy fields are present.

AC-3:
Given Python legacy backend adapter reason envelopes, when normalized, then deterministic reason-code
mapping remains regression-pinned.

AC-4:
Given SDK docs checks, when taxonomy/evidence marker surfaces drift, then checks fail closed.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
  - Expectation: deterministic taxonomy marker set present on pass path.

- C-02 (AC-2, Integration):
  - Test: `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
  - Expectation: deterministic live taxonomy/evidence marker set present on pass path.

- C-03 (AC-3, Unit/Regression):
  - Test: `python3 -m unittest tests.python.test_sdk`
  - Expectation: deterministic legacy reason normalization cases are stable.

- C-04 (AC-4, Docs):
  - Tests:
    - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
    - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
  - Expectation: docs marker surfaces are pinned.

## Success Metrics / Observable Signals

- Stable SDK HTTP taxonomy and evidence output markers across runs.
- Deterministic Python legacy reason normalization for adapter error fallback.
