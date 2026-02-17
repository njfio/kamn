# Plan — #4299

Status: Reviewed

## Approach

- Add a new deterministic checker (`scripts/ci/check_transport_observability_tls_ci_smoke_convergence.py`) that validates:
  - CI-fast workflow/local-heavy exclusions,
  - ci-tools fast-mode composition commands for transport/observability/TLS,
  - CI strategy + production plan marker parity,
  - bounded CI smoke max-seconds thresholds.
- Add RED-first regression tests in `scripts/ci/test_check_transport_observability_tls_ci_smoke_convergence.sh` using temporary tampered fixtures.
- Integrate the new test into `scripts/ci/test_ci_tools.sh` fast/full paths.
- Update docs and docs-contract tests for new convergence marker section.

## Affected Areas

- `scripts/ci/check_transport_observability_tls_ci_smoke_convergence.py`
- `scripts/ci/test_check_transport_observability_tls_ci_smoke_convergence.sh`
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `scripts/ci/test_production_service_next_steps_contract.sh`

## Risks and Mitigations

- Risk: command-surface drift in `test_ci_tools.sh` causes checker churn.
  - Mitigation: deterministic required command list and explicit reason codes.
- Risk: docs drift and checker drift diverge.
  - Mitigation: docs-contract assertions in Rust and shell docs tests.
- Risk: over-broad budget guard causes false failures.
  - Mitigation: explicit CI smoke threshold constants and deterministic overflow reason.

## Interfaces and Contracts

- Checker output markers:
  - `reason_taxonomy_version`, `reason_codes_csv`, `reason_codes_value`
  - `transport_observability_tls_ci_smoke_convergence_status`
  - `transport_observability_tls_ci_smoke_max_seconds`
  - `transport_observability_tls_local_heavy_max_seconds`
  - `ci_smoke_lane_cost_profile`, `local_heavy_lane_execution_mode`
