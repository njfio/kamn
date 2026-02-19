# Issue #3794 Tasks

- Issue: #3794
- Status: In Progress

## Ordered Tasks
- [x] T1 (Red): add failing retry/reconnect marker contract assertions across transport resilience lane/policy/contract-lane tests.
- [x] T2 (Green): emit and validate deterministic `retry_reconnect_marker_contract_status=verified` marker in run-lane + policy contracts.
- [x] T3 (Integration): propagate marker through contract-lane aggregation output and report payload.
- [x] T4 (Regression): update docs marker references and rerun transport resilience/test-contract suites.
- [ ] T5 (Verify): run shell guardrails, open mergeable PR, and close issue with DoD markers.

## Tier Mapping
- Unit: policy required-field/value checks for retry/reconnect marker contract status.
- Functional: lane dry-run marker emission checks.
- Integration: contract-lane output/report marker propagation checks.
- Regression: tamper/missing marker fail-closed behavior and docs marker parity.
- Performance: N/A (no new command path or runtime workload introduced).
