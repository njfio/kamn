# Issue #3959 Tasks

- Issue: #3959
- Status: Completed

## Ordered Tasks
- [x] T1 (Red): add fallback denylist regression test at runtime key-source policy gate.
- [x] T2 (Green): implement fallback env policy check + deterministic taxonomy marker.
- [x] T3 (Regression): run targeted unit/functional/integration/regression signer-policy tests.
- [x] T4 (Verify): update issue process log, closeout markers, and spec status.

## Tier Mapping
- Unit: key-source policy taxonomy marker assertions.
- Functional: strict fallback-env rejection behavior.
- Integration: strict managed-external policy path still passes.
- Regression: strict env-local rejection reason code remains stable.
- Performance: N/A (policy guard only; no throughput-path change).
