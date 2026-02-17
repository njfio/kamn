# Plan — Issue #4172

## Approach

1. Add a dedicated custody/rotation CI smoke governance section to `docs/ci/strategy.md` with:
   - checker/test commands
   - deterministic taxonomy markers and reason-code list
   - explicit local-heavy exclusion policy
2. Add a dedicated R27.20 closure section to
   `docs/plans/2026-02-14-production-service-next-steps.md` with:
   - active issue chain for closure lineage
   - convergence status, taxonomy marker, and CI/local-heavy boundary markers
   - checker/test command lineage
3. Extend `scripts/ci/test_production_service_next_steps_contract.sh` required marker list to
   include the new R27.20 section markers so drift fails closed.

## Affected Modules

- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- `scripts/ci/test_production_service_next_steps_contract.sh`

## Risks / Mitigations

- Risk: docs checker drift due to broad prose changes.
  Mitigation: enforce stable marker keys and command strings in docs-contract tests.
- Risk: inconsistency between strategy and plan sections.
  Mitigation: centralize marker vocabulary in checker constants and mirror in docs sections.

## Interfaces / Contracts

- Strategy/plan marker contract keys:
  - `custody_rotation_ci_smoke_convergence_status=verified`
  - `custody_rotation_ci_smoke_reason_taxonomy_version=...`
  - `custody_rotation_ci_smoke_max_seconds=120`
  - `custody_rotation_local_heavy_max_seconds=900`
  - command markers for checker and test script
- Docs-contract enforcement:
  - `scripts/ci/test_production_service_next_steps_contract.sh`
  - `scripts/ci/test_check_custody_rotation_ci_smoke_convergence.sh`

## ADR

No ADR required. Documentation/test parity update only.
