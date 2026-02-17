# Spec — #4283 Task: Build Failover Evidence Convergence Checker

Status: Implemented
Priority: P1
Parent: #4280
Milestone: R27.28 Live-node drift detection and failover-readiness governance

## Problem Statement

Failover-readiness governance currently validates preflight report markers and policy checks, but it does not enforce deterministic evidence convergence between drift-lane outputs and promotion decision artifacts.

## Scope

In scope:
- Deterministic convergence checker for failover preflight summary and promotion-policy artifacts.
- Deterministic fail-closed reason mapping when artifact linkage or payload convergence drifts.
- Regression tests for missing artifact links and tamper scenarios.
- Docs and docs-contract updates for convergence markers.

Out of scope:
- Runtime failover orchestration redesign.
- External promotion orchestrator changes.

## Acceptance Criteria

AC-1: Checker validates required evidence links across failover drift summary and promotion-policy artifacts.

AC-2: Missing or tampered evidence fails closed with deterministic reason markers.

AC-3: Promotion decision reason mapping remains deterministic across repeated runs.

AC-4: Failover ops and release checklist docs include convergence marker contracts and commands.

## Conformance Cases

- C-01 (AC-1, Functional): valid preflight report and policy artifact pass convergence check.
- C-02 (AC-2, Regression): missing policy artifact linkage marker fails with deterministic link-missing reason.
- C-03 (AC-2, Regression): tampered policy payload fails with deterministic payload-tamper reason.
- C-04 (AC-3, Regression): tampered reason mapping fails with deterministic reason-mapping mismatch marker.
- C-05 (AC-3, Regression): repeated convergence checks over identical drift fixtures preserve deterministic reason ordering.
- C-06 (AC-4, Conformance): docs-contract tests enforce convergence command and marker references.

## Success Signals

- Convergence checker fails closed on linkage/payload drift.
- Promotion reason mapping remains deterministic and test-covered.
- Docs and checker markers stay synchronized via contract tests.
