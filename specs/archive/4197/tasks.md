# Tasks — #4197 Local Full-Stack Harness Taxonomy Drift and Runbook Divergence Red Tests

## Ordered Tasks
- T1 (Regression): extend `scripts/runtime/test_check_local_full_stack_integration_live_policy.sh` with taxonomy drift tamper tests.
- T2 (Regression): extend `scripts/runtime/test_validate_local_full_stack_integration_live_contract_lane.sh` with runbook parity and runbook-tamper divergence tests.
- T3 (Docs): add local full-stack harness taxonomy/runbook mapping section in `docs/deploy/kolme_devnet_ops.md`.
- T4 (Docs Contract): add assertions in `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`.
- T5 (Verify): run targeted shell tests and docs-contract tests.

## Test Tier Mapping
- Unit: N/A (script/docs contract scope)
- Functional: local full-stack policy checker pass/fail assertions
- Integration: local full-stack contract-lane composition test
- Regression: taxonomy drift tamper and runbook divergence tamper checks
- Performance: N/A (bounded ci-smoke path only)
