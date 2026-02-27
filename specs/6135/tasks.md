# Tasks: Issue #6135

## Ordered Tasks
- T1 (RED): Add tests that fail with current substring-based success detection for non-boolean/invalid `ok` payloads.
- T2 (Implementation): Introduce `json_optional_bool_field` and switch MCP success checks to parsed boolean semantics.
- T3 (GREEN): Update/extend probe tests to confirm valid `ok:true` remains accepted.
- T4 (Verify): Run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness --tests -- -D warnings`, and targeted/full crate tests.

## Tier Mapping
- Unit: T1, T2, T3
- Functional: T3
- Regression: T1, T3
- Conformance: T3
- Integration: N/A (single-module internal behavior)
