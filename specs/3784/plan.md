# Issue #3784 Plan

- Issue: #3784
- Status: Implemented

## Approach
1. Add a red docs-contract assertion for missing local observability artifact schema marker declarations.
2. Update local observability section in `docs/ci/strategy.md` with explicit artifact schema markers for summary/policy/contract-lane reports.
3. Re-run targeted docs-contract test and local observability lane shell contract tests.
4. Run lint and shell guardrails to verify no shell-surface regression.

## Risks and Mitigations
- Risk level: low
- Risks:
  - Artifact schema markers can drift from script expectations.
  - Docs and contract tests can diverge, weakening fail-closed guarantees.
- Mitigations:
  - Explicit schema marker declarations in docs.
  - Fail-closed assertions in Rust docs-contract tests.

## Interface Contract
- Documentation + docs-contract test surface only.
- No runtime API/protocol/wire-format changes.

## ADR
- Not required.
