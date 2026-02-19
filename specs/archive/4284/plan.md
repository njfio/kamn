# Plan — #4284

Status: Reviewed

## Approach

- Identify existing CI smoke checker surfaces for failover/sync drill markers and lane boundaries.
- Add RED tests first for:
  - failover marker drift rejection
  - heavy failover-lane exclusion enforcement in fast-gate scope
  - deterministic repeated mismatch ordering
- Implement checker updates with deterministic reason taxonomy markers.
- Update docs and docs-contract tests:
  - `docs/ci/strategy.md`
  - `docs/plans/2026-02-14-production-service-next-steps.md`
  - relevant docs tests in `crates/kamn-core/tests/*_docs.rs`

## Affected Areas

- `scripts/ci/*` failover smoke checker and test harness scripts
- `.github/workflows/ci-fast-gate.yml` (verification only unless changes required)
- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- docs-contract tests under `crates/kamn-core/tests`

## Risks and Mitigations

- Risk: CI checker/docs marker drift.
  - Mitigation: docs-contract assertions for required marker strings.
- Risk: checker broadens fast-gate scope unintentionally.
  - Mitigation: explicit heavy-lane exclusion checks and regression tests.

## Interfaces and Contracts

- CI smoke checker output will include deterministic marker drift + boundary reason markers.
- Fast-gate boundary contract remains smoke-only; heavy failover lanes remain local/scheduled-only.
