# Issue #4033 Spec - Deep-Scan Policy Checker and CI Dry-Run Governance

- Status: Reviewed
- Issue: #4033
- Parent: #4027
- Milestone: R27.11 Dependency, license, and supply-chain governance hardening

## Problem Statement
Issue `#4032` delivered a deterministic local-heavy deep dependency runner and artifact schema,
but governance still lacks a fail-closed policy checker for marker drift, docs parity, and
CI dry-run command-surface boundary enforcement.

## Scope
In scope:
- Add deep-scan policy checker for runner report marker/schema/profile validation.
- Add docs parity checks across `docs/ci/strategy.md` and `docs/ops/configuration.md` markers.
- Add CI dry-run governance checks for required/forbidden command-surface entries.
- Add checker contract tests and checker shell regression harness.

Out of scope:
- Executing deep-scan run-mode lane in ci-fast-gate.
- External scanner/SaaS integrations.

## Acceptance Criteria
- AC-1: Policy checker fails closed on marker/schema/profile/taxonomy drift.
- AC-2: CI dry-run governance validates required/forbidden deep-scan command-surface boundaries.
- AC-3: Unit, Functional, Integration, Regression, and Performance tests are present and passing.

## Conformance Cases
- C-01 (Unit, AC-1): policy checker accepts valid baseline deep-scan report with deterministic taxonomy markers.
- C-02 (Functional, AC-1): tampered deep-scan report fails closed with deterministic policy reason codes.
- C-03 (Integration, AC-2): CI dry-run governance markers require checker command and forbid run-mode leakage.
- C-04 (Regression, AC-1/AC-2): docs marker parity drift in strategy/ops fails checker deterministically.
- C-05 (Performance, AC-3): policy checker remains bounded under low-cost CI budget.

## Success Metrics
- Deep-scan policy checker returns deterministic pass/fail decisions with stable reason taxonomy.
- CI dry-run command-surface boundary drift is caught without executing deep-scan run mode.
- Docs marker parity is contract-tested against checker and runner constants.
