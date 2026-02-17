# Plan — #4411

Status: Reviewed

## Approach

- Extend `scripts/runtime/test_check_unified_api_observability_local_heavy_live_policy.sh` with deterministic RED fixtures that simulate run-mode partial evidence-link reports.
- Ensure fixtures set other run-mode fields valid so failures isolate evidence-link convergence gaps.

## Affected Areas

- `scripts/runtime/test_check_unified_api_observability_local_heavy_live_policy.sh`

## Risks and Mitigations

- Risk: red fixtures accidentally fail for unrelated reasons.
  - Mitigation: construct fixtures from known-good base report and mutate only evidence-link fields.

