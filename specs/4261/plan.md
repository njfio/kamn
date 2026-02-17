# Plan — #4261 Partition-Finality CI Smoke Checker

Status: Reviewed

## Approach

1. Create checker script from existing CI smoke convergence checker pattern.
2. Encode deterministic reason taxonomy/order.
3. Add fixture-based shell test for baseline + drift/leakage/budget failures.
4. Hook checker test into CI tool suite.

## Affected Surfaces

- `scripts/ci/check_partition_finality_ci_smoke_convergence.py`
- `scripts/ci/test_check_partition_finality_ci_smoke_convergence.sh`
- `scripts/ci/test_ci_tools.sh`
