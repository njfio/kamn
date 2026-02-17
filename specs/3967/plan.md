# Plan — Issue #3967

## Approach

1. Refresh combined shell-surface baseline fixture values from current measured metrics.
2. Add deterministic refresh-workflow markers to `docs/ci/strategy.md`.
3. Add/extend docs-contract test coverage for those new markers.
4. Run targeted generator/policy/docs tests to verify fail-closed behavior remains intact.

## Affected Modules

- `fixtures/ci/combined_shell_surface_trend_baseline.json`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks / Mitigations

- Risk: baseline refresh masks real growth without process clarity.
  Mitigation: add explicit trigger and command markers in docs; keep thresholds unchanged.
- Risk: docs drift from implemented workflow.
  Mitigation: enforce markers with deterministic docs-contract tests.

## Interfaces / Contracts

- Preserve policy checker taxonomy and reason codes:
  - `kamn.ci.combined-shell-surface-trend-policy-reason-taxonomy.v1`
  - `combined_shell_surface_shell_line_total_delta_fail_exceeded`
- Preserve baseline fixture schema:
  - `kamn.ci.combined-shell-surface-trend-baseline.v1`

## ADR

No ADR required. No dependency/protocol architecture changes.

