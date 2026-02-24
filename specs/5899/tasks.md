# Tasks: Issue #5899 - Immediate Security/Runtime Remediation (Production Blockers)

1. T1 (Conformance/RED): add/adjust tests for signer failure fail-closed behavior, fallback-key absence checks, signature taxonomy consistency, and structured JSON parser edge cases.
2. T2 (Implementation): remove hardcoded fallback private key constants and enforce explicit signer provisioning.
3. T3 (Implementation): remove deterministic fallback signature path on signer errors; return explicit error.
4. T4 (Implementation): correct signature algorithm/profile taxonomy constants and dependent assertions.
5. T5 (Implementation): replace targeted hand-rolled JSON field extraction with `serde_json` in touched runtime paths.
6. T6 (Implementation): add deterministic bounds/eviction to touched replay/state guard maps.
7. T7 (Conformance/GREEN): run targeted crate tests for kamn-core, kamn-mcp-server, and kamn-sdk.
8. T8 (Regression): run workspace fast gates (`cargo fmt --check`, strict clippy) and document AC->tests mapping in PR.
