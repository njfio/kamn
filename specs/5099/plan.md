# Issue #5099 Plan

- Issue: #5099
- Status: Implemented

## Approach
1. Add additive M3 projection input/output contracts that reference `ContentRetrievalRequest` and `ContentRetrievalScope`.
2. Implement a projection API on `DataLayerM3SearchCatalog` that:
   - executes existing blind-index search,
   - maps each message ID through caller-provided message->CID map,
   - builds validated retrieval requests via `ContentRetrievalRequest::new`.
3. Add explicit fail-closed M3 error variants for missing CID mapping and invalid retrieval request projection.
4. Add RED tests for success and fail-closed paths, then implement and rerun full regression/guardrails.

## Risks and Mitigations
- Risk level: high
- Risks:
  - Projection could accidentally alter existing search behavior.
  - Cross-module error mapping could become ambiguous.
- Mitigations:
  - Keep existing search APIs unchanged; add new projection API only.
  - Use deterministic M3-specific projection error variants.
  - Lock behavior with conformance and regression tests.

## Interface Contract
- Additive M3 projection types and method.
- No dependency, protocol, or wire-format changes.

## ADR
- Not required for this scoped integration task.
