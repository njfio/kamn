# R48.1 Spec-volume and coherence batching mitigation

## Milestone Summary
Mitigate the remaining R48 review gap around spec-volume growth by codifying explicit coherence issue-batching and spec-volume guardrail markers that can be regression-tested in docs contracts.

## Issue Hierarchy
- Task:
  - `#5463` - codify coherence batching contract and spec-volume guardrail baselines

## Source Artifacts
- `docs/review/gaps-and-issues-r48.md`
- `docs/planning/`
- `crates/kamn-core/tests/`

## Governance Markers
- `coherence_contract_batching_policy_schema_version=kamn.review.coherence-contract-batching-policy.v1`
- `spec_volume_guardrail_policy_schema_version=kamn.review.spec-volume-guardrail-policy.v1`
- `spec_volume_guardrail_target_status=monitor|improving|stable`
