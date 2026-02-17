# Spec — #4224 Task: Admission/Backpressure CI Smoke Governance

Status: Reviewed
Priority: P1
Parent: #4220
Milestone: R27.24 Async API concurrency and admission-backpressure governance

## Problem Statement

Admission/backpressure governance markers exist in service API axum contract outputs, but fast-gate needs a dedicated low-cost CI smoke checker that fails closed on marker drift and enforces heavy lane exclusion.

## Scope

In scope:
- Add a CI smoke convergence checker for admission/backpressure governance.
- Enforce deterministic fail-closed reason taxonomy for marker drift and budget violations.
- Enforce heavy service-api-axum run command exclusion from ci-fast-gate workflow and ci-tools fast mode.
- Update docs and docs-contract tests with checker marker parity.

Out of scope:
- Service API runtime behavior redesign.
- Enabling heavy load lanes in fast-gate.

## Acceptance Criteria

AC-1: CI smoke checker validates required admission/backpressure smoke composition in ci-tools fast mode.

AC-2: Checker fails closed when service-api-axum run command leaks into ci-fast-gate workflow or ci-tools fast mode.

AC-3: Strategy/plan docs contain deterministic marker taxonomy and boundary policy for admission/backpressure CI smoke governance.

AC-4: CI/docs contract tests cover pass path and deterministic fail reasons for drift and budget overflow.

## Conformance Cases

- C-01 (AC-1, Functional): baseline repo state returns `status=pass`, `final_decision=GO`, and `admission_backpressure_ci_smoke_convergence_status=verified`.
- C-02 (AC-1, Regression): missing required fast-mode smoke command fails with deterministic `*_ci_smoke_composition_missing` reason.
- C-03 (AC-2, Regression): leaked service-api-axum run command in ci-tools fast mode fails with deterministic leakage reason.
- C-04 (AC-2, Regression): leaked service-api-axum run command in ci-fast-gate workflow fails with deterministic exclusion reason.
- C-05 (AC-4, Regression): `--max-seconds` over policy fails with deterministic seconds-exceeded reason.
- C-06 (AC-3, Docs): docs contract tests fail closed when required checker markers drift.
