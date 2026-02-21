# Issue #4035 Spec - Regression + Remediation-Marker Coverage for License Metadata Mismatches

- Status: Reviewed
- Issue: #4035
- Parent: #4028
- Milestone: R27.11 Dependency, license, and supply-chain governance hardening

## Problem Statement
Issue #4034 added fail-closed license-policy/manifest mismatch detection, but remediation-marker parity is not yet enforced across strategy and ops docs, and multi-reason mismatch behavior is not regression-protected by Rust contract tests.

## Scope
In scope:
- Add regression coverage for deterministic mismatch/fail-closed outputs in workspace license-policy contracts.
- Add deterministic remediation marker blocks for dependency-license metadata governance in strategy + ops docs.
- Add docs-contract tests that fail closed when remediation markers drift from checker reason codes.

Out of scope:
- Legal-policy changes or license model changes.
- New checker reason codes beyond the existing taxonomy.

## Acceptance Criteria
- AC-1: License mismatch scenarios remain fail closed with deterministic reason markers, including mixed multi-reason mismatch combinations.
- AC-2: `docs/ci/strategy.md` and `docs/ops/configuration.md` include synchronized remediation markers for every dependency-license governance reason code.
- AC-3: Unit, Functional, Integration, and Regression tests are present and passing; Performance is N/A (docs/test-governance only, no runtime-path change).

## Conformance Cases
- C-01 (Unit, AC-1): workspace checker baseline remains deterministic for taxonomy + boundary markers.
- C-02 (Functional, AC-1): checker fails closed for root policy/manifest mismatch fixtures with deterministic reason codes.
- C-03 (Integration, AC-2): docs contract enforces strategy/ops remediation marker parity for each dependency-license reason code.
- C-04 (Regression, AC-1): multi-reason mismatch output remains deterministic (`reason_codes_csv` ordering + `reason_class=mixed`).
- C-05 (Regression, AC-2): docs-marker drift on dependency-license remediation entries fails docs-contract target.

## Success Metrics / Observable Signals
- Every dependency-license reason code has a matching `metadata_governance_remediation.<reason>=...` marker in strategy and ops docs.
- Rust docs-contract tests fail closed if any remediation marker is removed/renamed.
- Workspace checker regression test protects deterministic mixed-reason classification and marker ordering.
