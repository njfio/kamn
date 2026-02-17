# Plan — #4273 Taxonomy Enforcement + Runbook Parity Checks

Status: Reviewed

## Approach

1. Extend shared contract lane runner with optional runbook marker parity enforcement.
2. Configure service api axum ingress lane with deterministic runbook marker contract and reason categories.
3. Update deploy compatibility + release checklist docs and add docs-contract assertions.

## Risks

- Shared-runner regression risk.
  - Mitigation: keep feature optional and test axum lane path explicitly.
