# Issue 6261 Plan

## Approach
1. Replace `eval` execution patterns with explicit `bash -lc` command execution or structured parsing logic.
2. For `run_critical_path_mutation_gate.sh`, remove assignment `eval` by parsing key/value output without shell evaluation.
3. Refresh waiver metadata:
   - extend expiry dates beyond near-term window,
   - add explicit mitigation issue reference (`#6261`).
4. Run shell governance checks and script regression tests.

## Affected Paths
- `scripts/kolme/run_local_e2e_integration_lane_impl.sh`
- `scripts/kolme/run_local_bootstrap_health_checks_lane_impl.sh`
- `scripts/kolme/run_local_heavy_validation_matrix_lane_impl.sh`
- `scripts/ci/run_critical_path_mutation_gate.sh`
- `.ci/script-surface-budget-waiver.json`
- `.ci/fast-gate-budget-delta-waiver.json`

## Risks and Mitigations
- Risk: command behavior drift after removing eval.
  - Mitigation: preserve command strings and execution ordering; run regression script checks.
- Risk: waiver refresh lacks traceability.
  - Mitigation: include mitigation issue markers in waiver payloads.

## Interface/Contract Notes
- No production protocol/API changes.
- Shell governance markers in outputs must remain stable.
