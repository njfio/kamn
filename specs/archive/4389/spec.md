# Spec — #4389 Task: Durable State-Store Integrity Checks and Fail-Closed Persistence Evidence Gating

Status: Implemented
Priority: P1
Parent: #4385
Milestone: R27.35 Async API framework hardening, real peer transport, and durable state-store validation governance

## Problem Statement

Promotion flows must reject tampered or stale persistence evidence deterministically and expose auditable reason taxonomy markers while keeping CI smoke checks low-cost.

## Scope

In scope:
- Persistence live validation marker and reason-taxonomy contract hardening.
- Tamper/freshness fail-closed policy checks for persistence evidence.
- CI smoke/local-heavy boundary markers for persistence gating.
- Release checklist and CI strategy docs parity updates.

Out of scope:
- Storage engine redesign/migration.
- New persistence backend implementation.

## Acceptance Criteria

AC-1: Persistence live validation emits deterministic tamper/freshness integrity markers and reason taxonomy outputs.

AC-2: Policy checks fail closed when persistence evidence is tampered, incomplete, or stale.

AC-3: CI smoke/local-heavy boundary rules for persistence gates are deterministic and auditable.

AC-4: Unit/Functional/Integration/Regression coverage exists and passes for the new contracts.

## Conformance Cases

- C-01 (AC-1, Functional): persistence validation output contains deterministic taxonomy version and reason-code CSV markers.
- C-02 (AC-1, Integration): persistence validation JSON includes deterministic tamper/freshness marker fields.
- C-03 (AC-2, Regression): tampered persistence policy markers are rejected with deterministic mismatch reasons.
- C-04 (AC-2, Regression): incomplete/stale persistence evidence is rejected fail-closed.
- C-05 (AC-3, Functional): CI smoke/local-heavy boundary markers are present and consistent in outputs/docs.
- C-06 (AC-4, Integration): target script suites and docs-contract tests pass.

## Success Metrics

- Deterministic marker parity across script stdout, JSON reports, and docs contracts.
- No acceptance path for tampered/stale/incomplete persistence evidence.
