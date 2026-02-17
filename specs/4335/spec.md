# Spec — #4335 Subtask: Deterministic Checker + Reason Taxonomy Integration

Status: Reviewed
Priority: P1
Parent: #4328
Milestone: R27.31 Signal-safe daemon lifecycle, streaming observability, and runtime-decomposition governance

## Problem Statement

Observability endpoint payload contract violations need deterministic, auditable fail-closed responses with stable reason taxonomy for release governance.

## Scope

In scope:
- Deterministic payload checker for observability endpoint surfaces.
- Stable reason taxonomy constants and fail-closed response envelope.
- Integration of checker into endpoint response rendering path.

Out of scope:
- Additional endpoint routes or collector/exporter infrastructure.

## Acceptance Criteria

AC-1: Checker validates required fields per endpoint surface.

AC-2: Checker validates expected schema-version markers where applicable.

AC-3: Contract violations fail closed with stable taxonomy version + reason code.

AC-4: Docs/checklist and docs-contract tests reflect the new taxonomy markers.

## Conformance Cases

- C-01 (AC-1, Unit): valid rendered payloads for `/metrics`, `/healthz`, `/readyz`, `/metrics.stream` pass checker.
- C-02 (AC-2, Unit): schema-version mismatch returns deterministic schema-drift reason.
- C-03 (AC-3, Functional): fail-closed envelope exposes stable status/final decision/taxonomy/reason markers.
- C-04 (AC-4, Conformance): docs include checker taxonomy version and reason-pattern contract markers.

## Success Signals

- Endpoint contract checker emits deterministic outputs under all validated surfaces.
- Docs contract tests remain green and enforce marker presence.
