# Issue #3774 Tasks

- Issue: #3774
- Status: In Progress

## Ordered Tasks
- [x] T1 (Implement): complete shared HTTP retry/backoff task (`#3778`).
- [x] T2 (Implement): complete notifications reconnect pacing/taxonomy task (`#3779`).
- [x] T3 (Implement): complete transport resilience local-heavy lane/exclusion task (`#3780`).
- [x] T4 (Verify): run story-level combined retry/reconnect + transport resilience verification bundle.
- [ ] T5 (Closeout): merge closure PR and close story issue with status markers.

## Tier Mapping
- Unit: reconnect/reason-marker composition checks.
- Functional: reconnect pacing and exclusion command-surface checks.
- Integration: local-heavy transport lane and contract-lane checks.
- Regression: docs/policy taxonomy drift guards.
- Performance: N/A (story closeout only; no new runtime workloads).
