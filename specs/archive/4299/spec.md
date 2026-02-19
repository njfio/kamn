# Spec — #4299 Task: Low-Cost CI Smoke Gate Convergence for Transport, Observability, and TLS Contracts

Status: Implemented
Priority: P1
Parent: #4295
Milestone: R27.29 Observability, transport resilience, and TLS governance convergence

## Problem Statement

Release promotion confidence requires a single low-cost CI smoke checker that verifies transport,
observability, and TLS governance contract convergence, while keeping local-heavy paths explicit
opt-in only.

## Scope

In scope:
- CI smoke checker composition across transport/observability/TLS contract policy surfaces.
- Deterministic fail-closed reason mapping when any domain contract drifts.
- CI-smoke/local-heavy boundary governance markers and budget guards.
- Docs and docs-contract synchronization for convergence taxonomy/markers.

Out of scope:
- Executing local-heavy runtime drills in ci-fast-gate.
- Rewriting transport/observability/TLS lane implementations.

## Acceptance Criteria

AC-1: Composite CI smoke checker fails closed on transport/observability/TLS contract mismatch.

AC-2: Composite checker reason outputs deterministically identify the mismatched contract domain.

AC-3: CI smoke budget thresholds and local-heavy opt-in boundaries are enforced.

AC-4: CI strategy and production-service closure docs are synchronized with convergence markers.

## Conformance Cases

- C-01 (AC-1, Functional): checker passes when required transport/observability/TLS CI smoke composition markers are present.
- C-02 (AC-1, Regression): checker fails with deterministic transport domain reason when transport composition marker drifts.
- C-03 (AC-1/AC-2, Regression): checker fails with deterministic observability domain reason when observability composition marker drifts.
- C-04 (AC-1/AC-2, Regression): checker fails with deterministic TLS domain reason when TLS composition marker drifts.
- C-05 (AC-3, Functional/Performance): checker fails closed on CI smoke max-seconds overflow.
- C-06 (AC-3, Regression): checker fails when local-heavy command leakage appears in fast-mode surfaces.
- C-07 (AC-4, Conformance): docs-contract tests assert convergence taxonomy/markers in CI strategy and production plan docs.

## Success Signals

- A single deterministic checker output surface captures transport/observability/TLS smoke convergence status.
- CI fast mode remains low-cost and excludes local-heavy commands by policy.
- Docs and checker marker taxonomy remain parity-aligned.
