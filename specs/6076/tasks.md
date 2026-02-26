# Tasks: Issue #6076

## Ordered Tasks
- T1 (Conformance): Map story AC-1..AC-4 to concrete runtime/service tests delivered under #6077.
- T2 (Implementation): Rebase and stabilize merged delivery tests against current `main` semantics where needed.
- T3 (Verification): Run `cargo fmt --check`, `cargo clippy -p kamn-node -- -D warnings`, `cargo test -p kamn-node`, and in-diff mutation gate.
- T4 (Closure): Update lifecycle artifacts and issue status markers for story completion.

## Tier Mapping
- Unit: T2
- Functional: T1, T2
- Integration: T1, T3
- Regression: T2, T3
- Conformance: T1, T3
- Mutation: T3
