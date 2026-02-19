# Spec — #4297 Task: Standardize API/Runtime/Kolme Correlation Fields and Enforce Structured Observability Schema Checks

Status: Implemented
Priority: P1
Parent: #4294
Milestone: R27.29 Observability, transport resilience, and TLS governance convergence

## Problem Statement

Unified API-observability local-heavy policy currently validates transport and evidence wiring, but it
does not enforce a deterministic correlation-field schema across API/runtime/Kolme surfaces. Drift can
pass silently and reduce diagnosability for release go/no-go decisions.

## Scope

In scope:
- Deterministic correlation schema markers in unified API-observability local-heavy run-lane outputs.
- Policy enforcement for required correlation schema fields and cross-surface propagation parity.
- Deterministic fail-closed reason taxonomy for schema drift and propagation mismatch classes.
- Docs updates for observability schema and release go/no-go correlation evidence requirements.

Out of scope:
- External telemetry collector rollout.
- Runtime protocol redesign.
- New runtime lanes or executables.

## Acceptance Criteria

AC-1: Unified lane emits deterministic API/runtime/Kolme correlation schema markers and version identifiers.

AC-2: Policy checker fails closed when required correlation schema fields are missing, malformed, or mismatched.

AC-3: Policy checker fails closed when correlation-id propagation across API/runtime/Kolme markers is inconsistent.

AC-4: Observability and release governance docs describe required markers and deterministic drift reason codes.

## Conformance Cases

- C-01 (AC-1, Functional): dry-run report contains deterministic correlation schema version, required-field CSV, and API/runtime/Kolme correlation field markers.
- C-02 (AC-2, Regression): tampered schema-version marker is rejected with deterministic schema-drift reason code.
- C-03 (AC-2, Regression): tampered required-field marker (missing/invalid) is rejected with deterministic required-field reason code.
- C-04 (AC-3, Regression): tampered API/runtime/Kolme correlation-id parity is rejected with deterministic propagation-mismatch reason code.
- C-05 (AC-4, Conformance): `docs/observability/schema.md` and `docs/foundation/release-gonogo-checklist.md` include correlation schema markers and fail-closed reason taxonomy strings.

## Success Signals

- Unified local-heavy policy rejects correlation schema drift deterministically.
- Correlation parity mismatches are surfaced with stable reason codes.
- Docs and docs-contract tests remain synchronized with checker behavior.
