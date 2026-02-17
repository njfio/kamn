# Plan — #4306

Status: Reviewed

## Approach

- Implement a Python checker with deterministic reason taxonomy and ordered reason codes.
- Validate workflow + ci-tools fast-mode + docs marker parity in one pass.
- Add regression tests using temporary tampered fixtures to force deterministic failures.
- Integrate checker test into ci-tools fast and full modes.

## Affected Areas

- `scripts/ci/check_transport_observability_tls_ci_smoke_convergence.py`
- `scripts/ci/test_check_transport_observability_tls_ci_smoke_convergence.sh`
- `scripts/ci/test_ci_tools.sh`

## Risks and Mitigations

- Risk: false positives from command formatting differences.
  - Mitigation: exact required command markers and stable grep/substring checks.
- Risk: unstable reason ordering.
  - Mitigation: fixed reason-code ordering and normalized serialization.

## Interfaces and Contracts

- Report schema: `kamn.ci.transport-observability-tls-ci-smoke-convergence-report.v1`
- Taxonomy marker: `kamn.ci.transport-observability-tls-ci-smoke-convergence-reason-taxonomy.v1`
