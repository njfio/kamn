# Spec: Issue #4347

Status: Implemented
Issue: #4347
Parent: #4342
Milestone: R27.32 Script-surface consolidation, documentation graduation, and architecture-navigability governance
Priority: P1

## Problem Statement

Current trend tests do not fail when taxonomy markers are missing and do not explicitly validate
runtime-budget overrun fail-closed behavior.

## Scope

In scope:
- RED checks for taxonomy marker presence.
- RED checks for deterministic runtime budget overrun reason-code behavior.

Out of scope:
- Implementation of taxonomy/budget features (handled by #4348).

## Acceptance Criteria

AC-1:
Given trend checker pass/fail outputs, when taxonomy markers are absent, then tests fail
non-flakily.

AC-2:
Given max-runtime budget checks, when budget is exceeded, then tests fail with deterministic reason
codes.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `bash scripts/ci/test_check_non_kolme_wave19_wrapper_family_budget_trend.sh`
  - Expectation: fails RED before taxonomy marker implementation.

- C-02 (AC-2, Regression):
  - Test: `bash scripts/ci/test_check_non_kolme_wave19_wrapper_family_budget_trend.sh`
  - Expectation: fails RED before runtime budget reason mapping implementation.

## Success Metrics / Observable Signals

- RED failures clearly describe missing taxonomy and runtime-budget contracts.
