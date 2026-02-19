# Issue #4373 Tasks

Status: Reviewed

T1 (RED tests first) [#4379]:
- Add failing script assertions for simulated-signature acceptance and missing native signer taxonomy output markers.

T2 (GREEN implementation) [#4380]:
- Implement native signer fail-closed checks and deterministic taxonomy outputs in signed-to-Kolme policy checker.
- Wire deterministic runtime signing profile evidence fields in signed-to-Kolme contract-lane summary.

T3 (Docs + docs contract updates):
- Update release/devnet/readme markers for native signer taxonomy and rejection behavior.
- Update Rust docs contract tests to assert new marker presence.

T4 (Verification):
- Run targeted script tests and docs tests.
- Run `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`.

T5 (Delivery):
- Commit spec/tests/impl/docs changes atomically.
- Open PR with AC/test-tier/TDD evidence, ensure CI green, merge, and close #4373/#4379/#4380.
