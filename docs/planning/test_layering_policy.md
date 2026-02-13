# Test Layering Policy

This document defines how KAMN distributes validation across test layers while keeping PR CI cost bounded.

## Contract Markers

- `policy_schema_version=kamn.test-layering-policy.v1`
- `unit_hotspots_required=true`
- `integration_coverage_reduction_allowed=false`
- `ci_fast_gate_cost_budget_required=true`
- `layering_drift_contract=enabled`

## Layering Rules

1. Every production hotspot must have direct unit coverage in addition to integration coverage.
2. Integration suites are not reduced to satisfy unit-test targets.
3. Policy drift checks must fail closed when required markers disappear.
4. Fast-gate checks must remain bounded and low-cost.

## Domain Matrix

| Domain | Unit | Functional | Integration | Notes |
| --- | --- | --- | --- | --- |
| runtime-kolme-live | required | required | required | keep retry/provider drift failures localizable |
| signer-secret-handling | required | required | required | keep fail-closed key handling regressions localized |
| ci-policy-contracts | required | required | required | keep policy drift deterministic |

## Evidence

- Local contract command: `bash scripts/ci/test_check_test_layering_policy.sh`
- Policy checker command:
  - `python3 scripts/ci/check_test_layering_policy.py --policy-doc docs/planning/test_layering_policy.md --strategy-doc docs/ci/strategy.md --output-json /tmp/test-layering-policy-report.json`

## Regression

- Regression: #2694
