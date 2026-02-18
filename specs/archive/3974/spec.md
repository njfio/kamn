# Spec — Issue #3974

- Title: Subtask: graduate first missing-docs module batch with complete public API doc coverage
- Parent: #3968
- Milestone: R27.7 Script-surface consolidation and docs graduation
- Status: Implemented
- Priority: P1

## Problem Statement

The first graduation batch (`bootstrap`, `key_recovery`, `kolme_runtime_commit`) is reflected in docs/fixtures, but there is no explicit contract that these modules must remain present in the graduated fixture and protected from allowlist reintroduction as a named batch.

## Objective

Add explicit first-batch graduation contracts that fail closed when any batch module disappears from graduated fixtures or is reintroduced to missing-docs exemptions.

## Scope

In scope:
- Enforce first-batch module presence in graduated fixture checks.
- Add regression tests for first-batch fixture drift and allowlist bypass.
- Document first-batch regression contract markers in CI strategy docs.

Out of scope:
- Additional module-batch graduation waves.
- New missing-docs policy taxonomy beyond first-batch contract enforcement.

## Acceptance Criteria

- AC-1: Missing-docs policy checker fails when first-batch module set is not present in graduated fixture.
- AC-2: Missing-docs policy checker/tests fail when any first-batch module is reintroduced to allowlist exemptions.
- AC-3: Contract tests explicitly cover first-batch fixture drift and allowlist bypass regressions.
- AC-4: CI strategy docs describe first-batch graduation regression contract behavior.

## Conformance Cases

- C-01 (AC-1): Removing `bootstrap` from graduated fixture fails missing-docs policy contract.
- C-02 (AC-1): Removing `key_recovery` from graduated fixture fails missing-docs policy contract.
- C-03 (AC-1): Removing `kolme_runtime_commit` from graduated fixture fails missing-docs policy contract.
- C-04 (AC-2): Re-adding `bootstrap` to allowlist fails contract.
- C-05 (AC-2): Re-adding `key_recovery` to allowlist fails contract.
- C-06 (AC-2): Re-adding `kolme_runtime_commit` to allowlist fails contract.
- C-07 (AC-3): `test_check_kamn_core_missing_docs_policy.sh` covers C-01..C-06.
- C-08 (AC-4): `docs/ci/strategy.md` documents first-batch regression contract path and module set.

## Success Metrics

- First-batch graduation state cannot regress silently in fixtures or allowlist.
- CI contract checks provide deterministic batch-level failure on first-batch drift.
