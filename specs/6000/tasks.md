# Tasks: Issue #6000

## Ordered Tasks
- T1 (RED): Add failing tests asserting new writes use `kamn:cid:v2` + `sha256:` and that tampering fails verification.
- T2 (RED): Add failing compatibility tests proving legacy `kamn:cid:v1` payloads still load and verify.
- T3 (Implementation): Replace FNV-only CID/tag derivation with versioned SHA-256 default plus legacy compatibility.
- T4 (GREEN): Run targeted `kamn-core` content-storage tests and ensure all new conformance cases pass.
- T5 (Regression): Run adjacent content/message integrity tests to confirm no behavioral regressions.

## Tier Mapping
- Unit: T1, T3, T4
- Functional: T1, T3, T4
- Integration: T2, T4
- Regression: T5
- Conformance: T4
