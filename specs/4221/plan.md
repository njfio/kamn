# Plan — #4221

Status: Implemented

## Approach
- Identify current async API concurrency contract scripts and marker surfaces.
- Add/extend deterministic checker logic for in-flight + queue budget contracts.
- Add red tests first for mismatch/tamper scenarios (from #4225), then implement checker outputs (from #4226).
- Integrate marker propagation into lane output and CI/docs contract surfaces.

## Affected Modules
- `scripts/runtime/*async*` or corresponding service-api concurrency lane/checker scripts.
- `scripts/ci/test_ci_tools.sh` fast/full composition.
- Docs contract surfaces in `docs/ci/strategy.md`, `docs/foundation/release-gonogo-checklist.md`, and supporting planning docs.
- Rust docs-contract tests under `crates/kamn-core/tests/*docs.rs`.

## Risks / Mitigations
- Risk: marker-name drift across checker/lane/docs/tests.
  - Mitigation: centralize constants and enforce via docs-contract tests.
- Risk: CI runtime growth.
  - Mitigation: keep checks in low-cost smoke path, keep heavy runs local opt-in.

## Interfaces / Contracts
- Deterministic reason taxonomy/version/csv markers for concurrency budget outcomes.
- Fail-closed reason marker conventions for tamper/mismatch cases.
