# Issue #3932 Plan

- Issue: #3932
- Status: Implemented

## Approach
1. Add a red docs-contract assertion in `ci_strategy_docs.rs` for invariant-fuzz-concurrency CI-smoke/local-heavy governance markers.
2. Update `docs/ci/strategy.md` with a dedicated section that declares:
   - bounded CI-smoke command path
   - explicit local-heavy opt-in path
   - exclusion from `ci-fast-gate` defaults for heavy mode
   - deterministic policy command markers
3. Re-run docs-contract + invariant-fuzz-concurrency script tests and lint gates.
4. Package issue closure evidence with shell-surface DoD markers.

## Risks and Mitigations
- Risk level: low
- Risks:
  - Documentation phrasing drift could break contracts unexpectedly.
  - Incomplete marker coverage could leave CI governance ambiguous.
- Mitigations:
  - Use explicit marker strings and corresponding fail-closed assertions.
  - Keep scope confined to docs/tests/spec files.

## Interface Contract
- Documentation and docs-contract tests only.
- No production API/runtime behavior changes.

## ADR
- Not required (governance docs-contract hardening only).
