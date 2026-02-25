# Tasks: Issue #5931 - Task: Harden managed signer execution and secret env handling

- Issue: #5931
- Spec: `specs/5931/spec.md`
- Plan: `specs/5931/plan.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Ordered Tasks
- T1 (RED / Conformance): added failing security regressions for command-injection payload and signer-secret env leakage in managed signer backend tests.
- T2 (GREEN / Implementation): replaced `sh -c` subprocess invocation with argv-tokenized direct spawn and explicit child env scrubbing.
- T3 (Refactor): centralized managed-signer command parsing + child env allowlist handling in `managed_backend.rs`.
- T4 (Regression): ran managed backend unit slice and `main_tests::signer_tests` managed-external matrix.
- T5 (Verify): ran `cargo fmt --check`, strict `cargo clippy -p kamn-node --bin kamn-node -- -D warnings`, plus docs contract tests tied to updated signer docs.
- T6 (Process): updated spec/plan/tasks and signer runtime docs to reflect hardened managed signer execution contract.
