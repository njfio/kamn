# Issue #5236 Tasks

- Issue: #5236
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Ordered Tasks
- T1 (Tests/RED): reproduce CI failure for `block_pipeline` and `block_pipeline_transport_fed` with current fixtures.
- T2 (Implementation/GREEN): update invalid listener DID test fixtures in `block_pipeline.rs` to typed DID-compliant values.
- T3 (Implementation/GREEN): update invalid listener DID test fixtures in `block_pipeline_transport_fed.rs` to typed DID-compliant values.
- T4 (Regression): run both failing targets together and verify deterministic green.
- T5 (Verification): run shell-ratio guardrail and record shell-surface actual markers.
- T6 (Process): update issue status/closure artifacts and set spec status to `Implemented` after merge.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | N/A (fixture-only regression correction) |
| Functional | block-pipeline commit-path behavior remains green |
| Integration | transport-fed fork-choice/stale/ordering cases remain green |
| Regression | previously failing CI targets pass in one run |
| Performance | existing local budget assertion in transport-fed suite |
