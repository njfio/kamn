# Tasks — #4237 Replay Idempotency Taxonomy and Runbook Marker Parity

Status: Reviewed

- T1 (Regression): add red tests for replay taxonomy drift and runbook marker divergence
  (`#4242`).
- T2 (Implementation): add replay taxonomy enforcement + runbook parity checks in sqlite
  crash-recovery policy and contract lane (`#4243`).
- T3 (Docs): update deploy/release marker declarations and Rust docs-contract assertions.
- T4 (Verification): run targeted sqlite runtime policy/contract-lane + docs tests.
