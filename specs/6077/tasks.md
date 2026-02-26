# Tasks: Issue #6077

## Ordered Tasks
- T1 (RED): Add regression test proving daemon does not synthetically project `created -> relayed` when no recipient forward route is available.
- T2 (RED): Add/extend live integration test for send->failed-forward(requeue)->successful-forward->recipient-delivered across restart.
- T3 (Implementation): Update daemon relay tick logic to project status only for successfully forwarded entries and preserve pending spool entries otherwise.
- T4 (GREEN): Run scoped runtime and service API integration tests validating durable retry and recipient-visible delivery.
- T5 (Docs): Add `docs/architecture/service-api-delivery-flow.md` and describe durable lifecycle, retry semantics, and restart behavior.
- T6 (Regression): Re-run relay projection and cross-node delivery regression lanes.

## Tier Mapping
- Unit: T3
- Functional: T1, T3, T4
- Integration: T2, T4
- Regression: T1, T2, T6
- Conformance: T4, T6
- Performance: N/A (no throughput budget change in this task)
