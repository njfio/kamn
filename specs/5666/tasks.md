# Tasks: #5666 Enable cargo-mutants In-Diff Gate for Portable-Agent Slices

- [x] T1 (Conformance/Functional): capture RED evidence showing `cargo mutants` command is unavailable.
- [x] T2 (Implementation): install `cargo-mutants` and capture GREEN invocation evidence.
- [x] T3 (Docs/Conformance): update `docs/ci/strategy.md` with install, in-diff invocation, and fallback behavior.
- [x] T4 (Regression): re-run `cargo mutants --in-diff --list` to validate stable invocation.
- [x] T5 (Verify): run `cargo fmt --all --check` and targeted docs checks if required.
- [x] T6 (Closure): set spec status to Implemented and close issue with command evidence.
