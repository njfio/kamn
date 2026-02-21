# Issue #4082 Spec - CI Dry-Run Tamper Checker and Go/No-Go Marker Parity Contracts

- Status: Implemented
- Issue: #4082
- Parent: #4074
- Milestone: R27.14 Data lifecycle, retention, and privacy control hardening

## Problem Statement
Lifecycle artifact integrity evidence now exists, but CI-fast governance lacks a dedicated
fail-closed checker that validates lifecycle tamper markers and release go/no-go marker
parity against deterministic contracts.

## Scope
In scope:
- Add a CI dry-run lifecycle tamper governance checker that evaluates lifecycle artifact
  integrity bundle markers and go/no-go dry-run marker parity.
- Add deterministic threshold fixture and fast-mode selector/workflow exclusion parity checks.
- Add docs parity enforcement markers across `docs/ci/strategy.md` and
  `docs/ops/configuration.md`.
- Add unit, functional, integration, regression, and performance tests for the checker.

Out of scope:
- Local-heavy run-mode execution in CI fast-gate.
- External notarization or third-party attestation.

## Acceptance Criteria
- AC-1: Checker fails closed when lifecycle artifact markers or go/no-go dry-run markers drift.
- AC-2: Checker enforces deterministic release go/no-go marker parity contracts and emits
  stable reason taxonomy outputs.
- AC-3: CI fast-mode selector and workflow exclusion parity for lifecycle governance remain
  validated.
- AC-4: Unit, Functional, Integration, Regression, and Performance coverage is present and green.

## Conformance Cases
- C-01 (Unit, AC-1): checker accepts baseline lifecycle bundle + go/no-go dry-run report and
  emits `status=pass` with deterministic taxonomy/version markers.
- C-02 (Functional, AC-1): tampered lifecycle bundle marker fails checker with deterministic
  `lifecycle_ci_dry_run_lifecycle_marker_parity_drift`.
- C-03 (Integration, AC-2/AC-3): leaked forbidden go/no-go run-mode command in workflow fails
  checker with deterministic `lifecycle_ci_dry_run_workflow_exclusion_drift`.
- C-04 (Regression, AC-2): docs remediation marker drift fails checker with deterministic
  `lifecycle_ci_dry_run_docs_remediation_marker_missing`.
- C-05 (Performance, AC-4): checker execution remains within bounded CI budget from threshold fixture.

## Success Metrics
- CI checker output remains deterministic (`status`, `final_decision`, taxonomy/version/reason CSV).
- Docs and threshold fixtures stay synchronized with checker contracts.
- CI fast-mode can verify lifecycle tamper governance without invoking local-heavy run-mode paths.
