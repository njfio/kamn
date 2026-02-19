# Issue #3783 Plan

- Issue: #3783
- Status: Reviewed

## Approach
1. Add a red docs-contract test suite for tracing taxonomy markers in `docs/observability/contracts.md`.
2. Add a tracing taxonomy section in docs with explicit version, field vocabulary, event markers, and drift reason markers.
3. Add integration-style parity assertions against runtime source files (`daemon_phase.rs`, `observability_endpoint.rs`, `logging.rs`).
4. Run target tests + lint + shell guardrails.

## Risks and Mitigations
- Risk level: low
- Risks:
  - Documentation changes may omit required markers and break drift protections.
  - Marker mismatch between docs and source can reduce trust in taxonomy contracts.
- Mitigations:
  - Fail-closed contract assertions for required markers.
  - Explicit parity checks against runtime source markers.

## Interface Contract
- Documentation and docs-contract tests only.
- No runtime behavior/API/wire-format changes.

## ADR
- Not required.
