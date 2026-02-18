# Plan — Issue #4187

## Approach

- Extend `scripts/ci/check_upgrade_compatibility_ci_smoke_convergence.py` to require `--plan-doc`
  and validate deterministic plan markers.
- Extend `scripts/ci/test_check_upgrade_compatibility_ci_smoke_convergence.sh` with plan-doc
  fixture coverage and baseline invocation updates.
- Update `docs/ci/strategy.md` upgrade compatibility section so command/taxonomy markers match the
  checker contract (including plan-doc drift reason).
- Add R27.21 closure sections and markers to:
  - `docs/plans/2026-02-14-production-service-next-steps.md`
  - `docs/planning/kolme-devnet-ops.md`
- Extend docs contract tests in:
  - `scripts/ci/test_production_service_next_steps_contract.sh`
  - `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`

## Affected Modules

- `scripts/ci/check_upgrade_compatibility_ci_smoke_convergence.py`
- `scripts/ci/test_check_upgrade_compatibility_ci_smoke_convergence.sh`
- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- `docs/planning/kolme-devnet-ops.md`
- `scripts/ci/test_production_service_next_steps_contract.sh`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`

## Risks and Mitigations

- Risk: checker/docs marker ordering drift breaks baseline unexpectedly.
  - Mitigation: keep deterministic marker csv order centralized in checker and mirrored in docs.
- Risk: docs tests become brittle to phrasing edits.
  - Mitigation: assert stable contract markers/commands, not broad prose sentences.

## Interfaces/Contracts

- Checker CLI extension:
  - `--plan-doc <path>` (required)
- New fail-closed reason code:
  - `production_plan_upgrade_compatibility_convergence_markers_missing`
- Deterministic closure markers (R27.21):
  - `upgrade_compatibility_ci_smoke_convergence_status=verified`
  - `upgrade_compatibility_ci_smoke_reason_taxonomy_version=kamn.ci.upgrade-compatibility-ci-smoke-convergence-reason-taxonomy.v1`
  - `upgrade_compatibility_ci_smoke_max_seconds=120`
  - `upgrade_compatibility_local_heavy_max_seconds=900`

## ADR

- Not required (docs/checker parity and contract test extension only).
