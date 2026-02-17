# Spec — #4410 Subtask: Deterministic Telemetry Failure Mapping + Normalized Emission Evidence Outputs

Status: Reviewed
Priority: P1
Parent: #4403
Milestone: R27.36 Deep validation hardening, concurrency safety, and observability-emission governance

## Problem Statement

Runtime observability policy checker output lacks normalized reason-value markers and does not map missing required fields through deterministic reason taxonomy.

## Scope

In scope:
- Deterministic reason mapping for missing required summary report fields.
- Normalized reason output marker emission in policy payload/CLI output.
- Contract-lane and docs parity updates.

Out of scope:
- Runtime endpoint business logic behavior changes.

## Acceptance Criteria

AC-1: Missing required summary fields map to deterministic fail-closed reason code(s).

AC-2: Policy outputs include normalized `reason_codes_value` on pass and fail paths.

AC-3: Existing drift/fail-closed mappings remain deterministic and regression-clean.

AC-4: Observability schema docs include updated taxonomy/normalized marker contract.

## Conformance Cases

- C-01 (AC-1, Functional): required field omission yields `runtime_observability_policy_required_field_missing:<field>`.
- C-02 (AC-2, Integration): policy output and JSON report contain `reason_codes_value`.
- C-03 (AC-3, Regression): tamper failures still map to deterministic existing reason codes.
- C-04 (AC-4, Docs): docs reflect runtime observability endpoint payload schema and reason markers.

