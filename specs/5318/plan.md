# Issue #5318 Plan

## Approach
1. Introduce `scripts/ci/performance_smoke_contracts.py` with two subcommands:
   - `generate`: current fixture lookup + provenance/seed validation + report JSON output.
   - `check`: current metric + marker validation against `.ci/performance-targets.env`.
2. Convert:
   - `scripts/ci/generate_performance_smoke_report.sh`
   - `scripts/ci/check_performance_thresholds.sh`
   into thin wrappers that delegate to the Python tool.
3. Preserve output strings and fail-closed reasons used by existing tests.
4. Run targeted script/Rust contract tests and collect LOC delta evidence.

## Affected Modules
- `scripts/ci/performance_smoke_contracts.py` (new)
- `scripts/ci/generate_performance_smoke_report.sh`
- `scripts/ci/check_performance_thresholds.sh`
- `specs/5318/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: subtle output/message drift breaks contract tests.
  - Mitigation: preserve existing reason strings and run existing contract tests unchanged.
- Risk: wrapper invocation path mismatch in CI.
  - Mitigation: keep shell entrypoint names/arguments stable; wrapper passes through all args.

## Interfaces and Contracts
- No workflow command changes.
- No schema changes to `fixtures/ci/performance_hot_path_fixture_matrix.json`.
- No threshold/env key changes in `.ci/performance-targets.env`.
