# Spec — #4328 Task: Streaming Observability Endpoint Schema Checks

Status: Reviewed
Priority: P1
Parent: #4325
Milestone: R27.31 Signal-safe daemon lifecycle, streaming observability, and runtime-decomposition governance

## Problem Statement

Observability endpoint payloads must be deterministically validated so schema drift and missing required fields cannot silently pass release governance.

## Scope

In scope:
- Deterministic checker validation for endpoint payload surfaces (`/metrics`, `/healthz`, `/readyz`, `/metrics.stream`).
- Stable reason taxonomy for schema drift and missing required endpoint fields.
- Docs + docs-contract tests that encode the checker taxonomy and fail-closed behavior.

Out of scope:
- External observability collector deployment.
- New endpoint surfaces beyond the current runtime observability endpoint routes.

## Acceptance Criteria

AC-1: Health/metrics/readiness/stream payload surfaces are checker-validated using deterministic required-field markers.

AC-2: Invalid payloads fail closed with stable reason-code taxonomy values.

AC-3: Contract tests cover missing-field and schema-drift scenarios with deterministic reason-code outputs.

AC-4: Observability schema docs and release go/no-go checklist include the checker taxonomy and required marker contract.

## Conformance Cases

- C-01 (AC-1, Unit/Functional): checker accepts valid health/metrics/readiness/stream payloads emitted by runtime renderer.
- C-02 (AC-2, Unit): checker rejects payloads missing required fields with `runtime_observability_policy_required_field_missing:<surface>.<field>` reason format.
- C-03 (AC-2, Unit): checker rejects schema version drift with `runtime_observability_policy_schema_drift:<surface>.schema_version` reason format.
- C-04 (AC-3, Regression): endpoint fail-closed error envelope carries stable taxonomy version and reason-code markers.
- C-05 (AC-4, Conformance): docs contract tests assert required taxonomy + fail-closed markers in observability and go/no-go docs.

## Success Signals

- Deterministic reason-code taxonomy emitted for all checker failures.
- No schema drift silently bypasses endpoint contract checks.
- `kamn-node` and `kamn-core` targeted test lanes pass with stable outputs.
