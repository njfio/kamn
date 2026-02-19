# Issue #5097 Plan

- Issue: #5097
- Status: Implemented

## Approach
1. Add deterministic M7 projection from telemetry-point records into `ObservabilitySample`.
2. Add owner-scoped M7 API that evaluates projected samples with `ObservabilityMonitor` and returns report/snapshot artifacts.
3. Reuse existing owner-scope authorization guard (`authorize_owner_scope`) for fail-closed cross-owner denial.
4. Add RED conformance tests for sample projection/evaluation and cross-owner denial.
5. Run scoped/full regression and shell guardrail evidence commands.

## Risks and Mitigations
- Risk level: high
- Risks:
  - Projection mapping could create invalid observability samples.
  - Integration could accidentally alter existing M7 aggregate/billing behavior.
- Mitigations:
  - Keep projection mapping deterministic and bounded with explicit validation handling.
  - Add targeted conformance tests and rerun full `kamn-core` regression suite.

## Interface Contract
- Additive M7 observability projection/evaluation APIs.
- No dependency, protocol, or wire-format changes.

## ADR
- Not required for this scoped integration refactor.
