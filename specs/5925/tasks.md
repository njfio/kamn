# Tasks: Issue #5925 - Task: Replace FNV-based key lifecycle audit chain hashing with cryptographic hash

- Issue: #5925
- Spec: `specs/5925/spec.md`
- Plan: `specs/5925/plan.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Ordered Tasks
- T1 (RED / Conformance): added failing conformance tests in `crates/kamn-core/tests/key_lifecycle.rs` for `sha256:v1` record-hash format and collision-style mutation behavior.
- T2 (GREEN / Implementation): migrated `KeyLifecycle` audit hash emission to SHA-256 with explicit version marker and retained verifier support for legacy v0 records.
- T3 (Refactor): simplified hash-version matcher to deterministic equality checks against computed v1 and legacy hashes.
- T4 (Regression): added `regression_issue_5925_rejects_unknown_record_hash_format` and verified docs-contract coverage.
- T5 (Verify): ran `cargo test -p kamn-core --test key_lifecycle -- --nocapture`, `cargo test -p kamn-core --test docs_contract_wave4_harness key_lifecycle_audit_trails_docs -- --nocapture`, `cargo fmt --check`, and strict clippy.
- T6 (Mutation): ran `cargo mutants --in-diff /tmp/issue5925.diff --package kamn-core --baseline skip -- --test key_lifecycle` with result `12 caught, 0 missed`.
- T7 (Process): updated issue status/log and set lifecycle artifacts to `Implemented`.
