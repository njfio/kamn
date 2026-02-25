# Tasks: Issue #5930 - Task: Implement HTTPS support in SDK service client and TLS validation

- Issue: #5930
- Spec: `specs/5930/spec.md`
- Plan: `specs/5930/plan.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Ordered Tasks
- T1 (RED / Conformance): added failing HTTPS fixture tests in `service_api_client.rs` for trusted CA success + fail-closed certificate/misconfiguration paths.
- T2 (GREEN / Implementation): replaced SDK `https://` NotImplemented branch with rustls transport and strict trust-chain validation.
- T3 (Refactor): introduced shared stream + TLS helper functions to unify HTTP and HTTPS request/read code paths.
- T4 (Regression): ran `cargo test -p kamn-sdk --test service_api_client` and `cargo test -p kamn-sdk`.
- T5 (Verify): ran `cargo fmt --check` and `cargo clippy -p kamn-sdk -- -D warnings`.
- T6 (Process): updated spec/plan/tasks + added ADR for dependency + TLS transport decision.
