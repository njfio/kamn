# Spec — Issue #3975

- Title: Subtask: add docs coverage contracts preventing missing-docs exemption regressions for graduated modules
- Parent: #3968
- Milestone: R27.7 Script-surface consolidation and docs graduation
- Status: Implemented
- Priority: P1

## Problem Statement

The missing-docs policy checker already blocks graduated modules from re-entering the allowlist, but exemption-regression failures do not emit deterministic policy reason markers. This weakens CI automation and governance documentation parity.

## Objective

Add deterministic missing-docs policy reason-code markers for graduated-module exemption regressions, wire tests for pass/fail marker behavior, and document the expanded reason taxonomy surface.

## Scope

In scope:
- Emit deterministic reason markers for graduated-module exemption regression failures.
- Expand missing-docs policy reason-code CSV taxonomy to include exemption-regression code.
- Add contract-test assertions for exemption-regression reason markers.
- Update CI/runtime governance docs and docs-contract tests with the updated reason-code CSV.

Out of scope:
- New missing-docs policy categories for unrelated lanes.
- Graduation of additional modules.

## Acceptance Criteria

- AC-1: Missing-docs policy checker emits deterministic taxonomy/reason-code markers for graduated-module exemption regressions.
- AC-2: Missing-docs policy reason-code CSV contains deterministic universe for rustdoc parity drift + exemption regression.
- AC-3: Contract tests validate exemption-regression marker emissions and updated reason-code CSV surface.
- AC-4: Governance docs and docs-contract tests remain synchronized with policy marker taxonomy.

## Conformance Cases

- C-01 (AC-1): Re-adding a graduated module to allowlist fails with `reason_code=graduated_module_exemption_regression`.
- C-02 (AC-1): Exemption-regression failure emits `reason_taxonomy_version=kamn.ci.kamn-core-missing-docs-policy-reason-taxonomy.v1`.
- C-03 (AC-2): Policy reason CSV equals `graduated_module_exemption_regression,rustdoc_navigation_parity_drift`.
- C-04 (AC-3): `test_check_kamn_core_missing_docs_policy.sh` asserts C-01/C-02/C-03.
- C-05 (AC-4): `docs/ci/strategy.md` + `docs/architecture/runtime.md` contain updated reason-code CSV marker and docs contract test remains green.

## Success Metrics

- Missing-docs exemption-regression failures become machine-parseable with deterministic reason markers.
- Missing-docs docs-governance marker contracts remain green in fast CI.
