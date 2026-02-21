# Issue #4032 Spec - Local-Heavy Deep Dependency Scan Runner and Artifact Schema

- Status: Reviewed
- Issue: #4032
- Parent: #4027
- Milestone: R27.11 Dependency, license, and supply-chain governance hardening

## Problem Statement
CI smoke dependency checks now have deterministic advisory + threshold contracts (`#4030`, `#4031`),
but local-heavy deep dependency scan execution lacks a deterministic runner artifact schema and
profile contract boundary for reproducible deep-scan evidence.

## Scope
In scope:
- Add deterministic local-heavy deep dependency scan lane runner contract.
- Enforce explicit local-heavy run-mode opt-in and ci-fast-gate boundary markers.
- Add deterministic profile fixture schema and runner projection markers.
- Add docs parity markers in `docs/ops/configuration.md` and docs-contract assertions.

Out of scope:
- Deep-scan policy checker and docs/runbook parity checker logic (follow-up `#4033`).
- Running deep-scan run-mode commands in ci-fast-gate.

## Acceptance Criteria
- AC-1: Runner emits deterministic artifact schema/version/taxonomy markers and profile projections.
- AC-2: Local-heavy run mode remains explicit opt-in and fail-closed without opt-in.
- AC-3: Unit, Functional, Integration, Regression, and Performance tests are present and passing.

## Conformance Cases
- C-01 (Unit, AC-1): deterministic schema/taxonomy markers and profile fixture metadata remain stable.
- C-02 (Functional, AC-1): injected-risk profile fails closed with deterministic reason code.
- C-03 (Integration, AC-2): run-mode execution requires explicit local-heavy opt-in and ci-fast-gate FAIL.
- C-04 (Regression, AC-1): fixture profile/schema contracts reject malformed columns/marker drift.
- C-05 (Performance, AC-3): dry-run execution remains bounded within local budget.

## Success Metrics
- Local-heavy deep dependency lane produces deterministic artifact schema markers and profile outputs.
- Run-mode boundary remains fail-closed without explicit opt-in.
- Ops docs marker block and docs-contract tests remain synchronized with runner contracts.
