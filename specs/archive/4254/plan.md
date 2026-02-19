# Plan — #4254 Partition-Finality CI Smoke Governance

Status: Reviewed

## Approach

1. Add `scripts/ci/check_partition_finality_ci_smoke_convergence.py` to validate:
   - fast-mode composition includes required smoke tests,
   - heavy run-mode commands remain excluded,
   - strategy/plan docs contain required deterministic markers.
2. Add `scripts/ci/test_check_partition_finality_ci_smoke_convergence.sh` with baseline and tamper fixtures.
3. Wire checker test into `scripts/ci/test_ci_tools.sh` fast/full mode suites.
4. Update `docs/ci/strategy.md` and `docs/plans/2026-02-14-production-service-next-steps.md` with new checker contracts.
5. Update docs-contract tests to lock marker parity.

## Affected Surfaces

- `scripts/ci/check_partition_finality_ci_smoke_convergence.py`
- `scripts/ci/test_check_partition_finality_ci_smoke_convergence.sh`
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks and Mitigations

- Risk: false positives from brittle fast-mode extraction.
  Mitigation: reuse existing deterministic fast-mode block parser pattern.
- Risk: docs drift over time.
  Mitigation: add explicit docs-contract assertions and checker-side marker checks.

## Interface and Contract Notes

- New checker CLI:
  - `python3 scripts/ci/check_partition_finality_ci_smoke_convergence.py --workflow-file ... --ci-tools-file ... --strategy-doc ... --plan-doc ... --max-seconds 120 --output-json <path>`
- Deterministic checker markers:
  - `partition_finality_ci_smoke_reason_taxonomy_version=...`
  - `partition_finality_ci_smoke_reason_codes_csv=...`
  - `partition_finality_ci_smoke_max_seconds=120`
  - `partition_finality_local_heavy_max_seconds=900`
  - `partition_finality_ci_smoke_lane_cost_profile=low`
  - `partition_finality_local_heavy_execution_mode=opt_in`

ADR: not required (no dependency/protocol architecture change).
