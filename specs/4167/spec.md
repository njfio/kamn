# Spec: #4167 Fallback Signer Prohibition and Explicit Signer-Material Requirement RED Tests

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

Fallback signer-key paths and implicit signer-material behavior can reappear unless explicit regression tests enforce fail-closed outcomes and stable reason mappings.

## Scope

In scope:
- Add focused failing-then-passing tests for fallback signer-key prohibition in deployment preflight policy checks.
- Add focused tests for explicit signer-material requirement enforcement reason mapping.
- Update operations docs to declare deterministic signer-config failure contracts.

Out of scope:
- Redesign of key distribution workflows.
- Runtime mode changes unrelated to signer configuration.

## Acceptance Criteria

AC-1 Tests fail when fallback signer paths are reachable.
AC-2 Tests fail when missing signer material does not produce explicit deterministic policy errors.
AC-3 Regression checks prevent fallback-path reintroduction.

## Conformance Cases

- C-01 (AC-1): when fallback signer secret is present in run-mode report, policy checker fails with `fallback_signer_secret_present_violation`.
- C-02 (AC-2): when signer material is missing in run-mode report, policy checker fails with deterministic signer-config reason mapping containing `signer_secret_missing`.
- C-03 (AC-3): docs and docs-contract tests include signer-config/fallback fail-closed markers and regression references.

## Success Metrics

- New signer-config fallback/missing-material cases are covered in deployment preflight checker tests.
- Existing preflight policy tests remain green.
- `docs/ops/configuration.md` includes deterministic signer-config contract markers validated by Rust docs-contract tests.
