# Plan — Issue #4157

- Status: Implemented

## Approach

- Add an R27.19 closure section to `docs/plans/2026-02-14-production-service-next-steps.md`
  with deterministic rehearsal/rollback governance markers, checker command references, and
  explicit CI vs local-heavy boundaries.
- Add a focused Rust docs-contract test that fails closed when those markers drift.

## Affected Modules

- `docs/plans/2026-02-14-production-service-next-steps.md`
- `crates/kamn-core/tests/rehearsal_rollback_governance_docs.rs`
- `specs/4157/spec.md`
- `specs/4157/plan.md`
- `specs/4157/tasks.md`

## Risks and Mitigations

- Risk: docs contract becomes brittle to prose rewrites.
  - Mitigation: assert stable marker keys/commands only, not free-form explanatory text.
- Risk: closure markers drift from checker taxonomy.
  - Mitigation: assert taxonomy/version/reason-codes markers tied to existing checker surfaces.

## Interfaces / Contracts

- Deterministic closure markers to enforce:
  - `rehearsal_promotion_ci_smoke_convergence_status=verified`
  - `rehearsal_promotion_ci_smoke_reason_taxonomy_version=kamn.ci.rehearsal-promotion-ci-smoke-convergence-reason-taxonomy.v1`
  - `rehearsal_promotion_ci_smoke_max_seconds=120`
  - `rehearsal_promotion_local_heavy_max_seconds=900`

## ADR

- Not required (docs + contract-test alignment only).
