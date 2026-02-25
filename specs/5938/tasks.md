# Tasks: Issue #5938 - Task: Expand fuzzing and property-based testing across parser/protocol surfaces

- Issue: #5938
- Spec: `specs/5938/spec.md`
- Plan: `specs/5938/plan.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Ordered Tasks
- T1 (RED / Conformance): derive failing tests from all C-xx conformance cases before implementation.
- T2 (GREEN / Implementation): implement in-scope behavior changes with minimal diff.
- T3 (Refactor): improve structure/readability while preserving green tests.
- T4 (Regression): run targeted module tests plus issue-specific regression suites.
- T5 (Verify): run cargo fmt --check, strict clippy for touched crates, and scoped tests to close ACs.
- T6 (Process): update docs/spec status and attach AC evidence in PR + issue closure.

## Execution Summary
- T1 completed: RED failure captured via `cargo test -p kamn-core --test cargo_fuzz_target_contract` with missing new corpus assets.
- T2 completed: Added `kolme_api_codec_parser` target plus corpus assets and metadata.
- T3 completed: Added parser/protocol proptest invariant suite with deterministic seed helper usage.
- T4 completed: Added corpus replay regression tests for new targets.
- T5 completed: Scoped formatting/lint/tests executed for touched crates.
- T6 completed: Updated docs (`docs/ci/strategy.md`, `docs/security/secure-coding.md`, `docs/architecture/README.md`, milestone index) and set lifecycle artifact statuses to Implemented.
