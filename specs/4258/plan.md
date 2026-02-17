# Plan — #4258 Finality Taxonomy Enforcement Implementation

Status: Implemented

## Approach

- Add deterministic finality taxonomy/runbook constants and marker projection in convergence lane.
- Add `--runbook-file` to policy checker and enforce required runbook markers.
- Add deterministic reason-code resolver for taxonomy/runbook drift projection.
- Wire new markers through contract lane report/stdout and doc surfaces.

## Risks

- Backward marker compatibility for existing tests.
  Mitigation: preserve existing markers/reasons and add new parity markers as additive contracts.
