# Spec — #4307 Subtask: Docs and Docs-Contract Tests for Transport-Observability-TLS Convergence Closure

Status: Reviewed
Priority: P1
Parent: #4299
Milestone: R27.29 Observability, transport resilience, and TLS governance convergence

## Problem Statement

Convergence checker and policy surfaces lose reliability if CI strategy and production closure docs drift
from actual marker/taxonomy contracts.

## Scope

In scope:
- CI strategy section for transport/observability/TLS smoke convergence marker surface.
- Production next-steps plan closure markers for R27.29 chain.
- Docs-contract assertions for both docs surfaces.

Out of scope:
- Roadmap reprioritization.

## Acceptance Criteria

AC-1: CI strategy docs include deterministic transport/observability/TLS convergence markers.

AC-2: Production next-steps doc includes R27.29 closure marker lineage.

AC-3: Docs-contract tests fail closed on marker drift.

## Conformance Cases

- C-01 (AC-1, Conformance): `ci_strategy_docs` assertions verify convergence section/markers.
- C-02 (AC-2, Conformance): production next-steps docs contract script verifies R27.29 markers.
- C-03 (AC-3, Regression): tampered doc fixture fails with deterministic marker-missing reason.

## Success Signals

- Doc and checker marker contracts remain synchronized in CI.
