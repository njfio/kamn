# Tasks: Issue #6145

## Ordered Tasks
- T1 (RED/Conformance): Add failing daemon relay tests for P2P message forwarding and requeue
  failure behavior.
- T2 (Implementation): Add daemon P2P relay config/context wiring and outbound P2P send path with
  HTTP fallback.
- T3 (Implementation): Add daemon inbound P2P inbox drain + service-api relayed-message upsert.
- T4 (GREEN/Regression): Keep/extend existing relay regressions (no-route, forward failure) to
  verify deterministic fallback semantics.
- T5 (Verification): Run scoped `kamn-node` unit/functional/regression/conformance commands and
  collect RED/GREEN evidence.
- T6 (Closure): Publish AC mapping, test-tier matrix updates, and measured closure evidence.

## Tier Mapping
- Unit: T1, T4, T5
- Functional: T2, T3, T5
- Integration: T1, T2, T3, T5
- Regression: T1, T4, T5
- Conformance: T1, T2, T3, T5, T6
