# Plan — #4292

Status: Reviewed

## Approach

- Update failover CI governance documentation in:
  - `docs/ci/strategy.md`
  - `docs/plans/2026-02-14-production-service-next-steps.md`
- Extend docs-contract tests to assert required marker/boundary strings.
- Keep docs marker strings aligned with checker outputs introduced in `#4291`.

## Affected Areas

- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- docs tests in `crates/kamn-core/tests`

## Risks and Mitigations

- Risk: brittle docs assertions due wording drift.
  - Mitigation: assert deterministic marker strings and command surfaces only.
