# Plan — Issue #4201

## Approach

- Introduce `scripts/ci/check_local_full_stack_integration_ci_smoke_convergence.py` that verifies:
  - required local full-stack CI smoke commands exist in ci-tools fast mode,
  - local full-stack run-mode commands are absent from fast-gate workflow and ci-tools fast mode,
  - strategy-document marker declarations remain synchronized,
  - max smoke budget guard (`--max-seconds <= 120`) remains enforced.
- Add deterministic shell contract tests in
  `scripts/ci/test_check_local_full_stack_integration_ci_smoke_convergence.sh`.
- Wire the new contract test into `scripts/ci/test_ci_tools.sh` in both fast/full paths.
- Update `docs/ci/strategy.md` with a dedicated local full-stack CI smoke convergence governance
  section.

## Affected Modules

- `scripts/ci/check_local_full_stack_integration_ci_smoke_convergence.py` (new)
- `scripts/ci/test_check_local_full_stack_integration_ci_smoke_convergence.sh` (new)
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`

## Risks and Mitigations

- Risk: checker drifts from command surface in future changes.
  - Mitigation: checker runs in ci-tools fast/full modes and fails closed on composition drift.
- Risk: false negatives when parsing ci-tools fast block.
  - Mitigation: reuse same fast-block extraction pattern already used by existing CI smoke checkers.
- Risk: policy-doc wording edits break marker matching.
  - Mitigation: keep required marker list explicit and deterministic in strategy doc.

## Interfaces/Contracts

- New checker CLI:
  - `--workflow-file <path>`
  - `--ci-tools-file <path>`
  - `--strategy-doc <path>`
  - `--max-seconds <int>`
  - `--output-json <path>` (optional)
- Output markers:
  - `status=pass|fail`
  - `final_decision=GO|NO-GO`
  - `reason_taxonomy_version=kamn.ci.local-full-stack-integration-ci-smoke-convergence-reason-taxonomy.v1`
  - `reason_codes_csv=<deterministic ordered csv>`
  - `reason_codes_value=none|<csv>`
  - `local_full_stack_ci_smoke_convergence_status=verified|violation`
  - `local_full_stack_ci_smoke_max_seconds=120`
  - `local_full_stack_local_heavy_max_seconds=900`
  - `local_full_stack_ci_smoke_lane_cost_profile=low`
  - `local_full_stack_local_heavy_execution_mode=opt_in`

## ADR

- Not required (bounded script-level policy checker addition without dependency/protocol changes).
