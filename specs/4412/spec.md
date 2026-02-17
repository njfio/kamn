# Spec — #4412 Subtask: Telemetry Reason Taxonomy and CI Smoke/Local-Heavy Boundary Governance

Status: Reviewed
Priority: P1
Parent: #4404
Milestone: R27.36 Deep validation hardening, concurrency safety, and observability-emission governance

## Problem Statement

Telemetry policy enforcement lacks deterministic reason taxonomy coverage for evidence-link convergence failures and explicit governance for CI smoke/local-heavy boundary evidence handling.

## Scope

In scope:
- Deterministic reason codes for telemetry evidence-link incompleteness and convergence drift.
- Policy checks that require complete run-mode evidence links and convergent linked artifacts.
- Run-lane evidence artifact preservation so policy links remain valid.
- Contract-lane and docs updates for deterministic governance markers.

Out of scope:
- New telemetry producers outside the existing unified local-heavy lane.

## Acceptance Criteria

AC-1: Policy checker rejects incomplete/partial run-mode evidence-link wiring with deterministic reason codes.

AC-2: Policy checker rejects non-convergent linked artifact contracts with deterministic reason codes.

AC-3: Pass/fail outputs emit stable reason taxonomy markers and normalized reason value fields.

AC-4: CI smoke/local-heavy boundary governance remains explicit in telemetry lane docs/outputs.

## Conformance Cases

- C-01 (AC-1, Functional): incomplete evidence-link maps fail closed with deterministic reasons.
- C-02 (AC-2, Integration): linked artifact schema/status drift fails closed with deterministic reasons.
- C-03 (AC-3, Functional): pass path emits deterministic taxonomy/version/csv/value markers.
- C-04 (AC-4, Docs): strategy docs include updated telemetry fail-closed marker set.

