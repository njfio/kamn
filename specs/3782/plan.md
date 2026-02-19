# Issue #3782 Plan

- Issue: #3782
- Status: Implemented

## Approach
1. Add startup logging configuration contract markers to `docs/observability/contracts.md` covering runtime modes, env controls, and deterministic fail-closed invalid-config markers.
2. Extend `crates/kamn-node/tests/observability_contracts_docs.rs` with docs-contract assertions for those markers and source alignment with `logging.rs`.
3. Re-run targeted runtime tests that verify startup logging behavior and invalid-config fail-closed behavior.
4. Run lint/guardrails to confirm no warnings and no shell-surface regression.

## Risks and Mitigations
- Risk level: medium
- Risks:
  - Docs and runtime source markers drift over time, weakening startup contract confidence.
  - Runtime mode startup marker semantics can regress without deterministic contract checks.
- Mitigations:
  - Fail-closed docs-contract assertions tied to runtime source marker strings.
  - Explicit conformance mapping to existing runtime integration/regression tests.

## Interface Contract
- Documentation + docs-contract tests only.
- No runtime API, protocol, or dependency changes.

## ADR
- Not required.
