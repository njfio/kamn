# Issue #3789 Plan

- Issue: #3789
- Status: Implemented

## Approach
1. Re-verify existing secure-mode TLS runtime tests and deterministic fail-closed markers in `observability_endpoint_tests.rs`.
2. Add missing issue specs (`spec.md`, `plan.md`, `tasks.md`) and map ACs to concrete test coverage.
3. Extend docs-contract tests to pin runtime-network TLS secure-mode env markers and negative-matrix reason taxonomy to source strings in `observability_endpoint.rs`.
4. Run targeted kamn-node test/lint/guardrail verification and close issue through merged PR.

## Risks and Mitigations
- Risk level: medium
- Risks:
  - TLS fail-closed markers could drift between docs and source, reducing operator confidence.
  - Route-serving secure-mode behavior could regress under refactors without explicit contract checks.
- Mitigations:
  - Fail-closed docs-contract assertions tied to runtime-network marker strings.
  - Targeted integration/regression coverage for TLS required-mode startup and negative matrix.

## Interface Contract
- Docs-contract + spec closure increment.
- No runtime API/protocol/dependency changes.

## ADR
- Not required.
