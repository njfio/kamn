# Issue #5418 Tasks

- T1 (Tests first): add phase-2 daemon-tests decomposition guard assertions and docs marker assertions.
- T2 (Implementation): extract topology contract tests into new include file and wire `daemon_tests.rs` shell marker/include.
- T3 (Docs): update `docs/ops/configuration.md` phase-2 decomposition markers.
- T4 (Verification): run targeted daemon/docs tests plus `cargo fmt --check` and `cargo clippy -- -D warnings`.
