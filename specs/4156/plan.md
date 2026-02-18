# Plan — Issue #4156

- Status: Implemented

## Approach

- Implement `scripts/ci/check_rehearsal_promotion_ci_smoke_convergence.py` by adapting the
  established CI-smoke convergence checker pattern used by existing governance lanes.
- Add `scripts/ci/test_check_rehearsal_promotion_ci_smoke_convergence.sh` with pass/fail fixtures
  for composition, exclusion, and docs-marker drift.
- Wire checker test into `scripts/ci/test_ci_tools.sh` fast-mode contract list.
- Add deterministic checker command and marker taxonomy entries to:
  - `docs/ci/strategy.md`
  - `docs/plans/2026-02-14-production-service-next-steps.md`

## Affected Modules

- `scripts/ci/check_rehearsal_promotion_ci_smoke_convergence.py`
- `scripts/ci/test_check_rehearsal_promotion_ci_smoke_convergence.sh`
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- `specs/4156/spec.md`
- `specs/4156/plan.md`
- `specs/4156/tasks.md`

## Risks and Mitigations

- Risk: marker drift between checker and docs causes false failures.
  - Mitigation: encode deterministic marker sets in checker and verify with contract fixtures.
- Risk: checker drift adds noisy failures for unrelated workflow edits.
  - Mitigation: scope checks to explicit required commands/markers only.

## Interfaces / Contracts

- Checker CLI:
  - `--workflow-file`
  - `--ci-tools-file`
  - `--strategy-doc`
  - `--plan-doc`
  - `--max-seconds`
  - `--output-json`
- Deterministic markers:
  - `rehearsal_promotion_ci_smoke_convergence_status=verified|violation`
  - `rehearsal_promotion_ci_smoke_reason_taxonomy_version=kamn.ci.rehearsal-promotion-ci-smoke-convergence-reason-taxonomy.v1`
  - `rehearsal_promotion_ci_smoke_max_seconds=120`
  - `rehearsal_promotion_local_heavy_max_seconds=900`

## ADR

- Not required (pattern-aligned checker + docs contract extension).
