# Tasks: Issue #5913 - Remove Compiled Fallback Signer Key Paths From kamn-core

1. T1 (RED): add failing regression tests for missing-key fail-closed behavior in signer backend and transaction key resolution.
2. T2 (GREEN): remove fallback key resolution usage in signer backend and transaction modules.
3. T3 (GREEN): update local-fallback integration tests to use explicit signer key env fixtures.
4. T4 (VERIFY): run targeted unit/integration lanes + `cargo fmt --check` + `cargo clippy -p kamn-core -- -D warnings`.
5. T5 (REGRESSION): run in-diff mutation lane for touched fallback-key removal logic.
