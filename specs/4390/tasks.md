# Tasks — #4390

Status: Reviewed

T1 (RED)
- Add failing tamper tests for concurrency/rate limit default drift.

T2 (RED)
- Add failing tamper tests for lifecycle taxonomy drift/missing fields and unstable backpressure projection status.

T3 (RED)
- Add normalized `reason_codes_value` marker assertions for both success and failure paths.

T4 (Regression)
- Re-run policy and contract-lane suites after #4391 implementation and verify deterministic reason outputs.
