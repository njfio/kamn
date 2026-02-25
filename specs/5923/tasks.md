# Tasks: Issue #5923 - Task: Replace deterministic agent-lib auth signatures with cryptographic signatures

- Issue: #5923
- Spec: `specs/5923/spec.md`
- Plan: `specs/5923/plan.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Ordered Tasks
- T1 (RED / Conformance): added four failing conformance tests in `auth_roundtrip` covering cryptographic parity, invalid-key rejection, deterministic forgery mismatch, and same-length tamper divergence.
- T2 (GREEN / Implementation): migrated `KamnAuthHeaders::build` from deterministic helper to cryptographic service-auth signing.
- T3 (Refactor): introduced SDK signing helper (`service_signature_for_state_hash_with_private_key`) and centralized service-auth -> SDK error mapping.
- T4 (Regression): ran `auth_roundtrip`, full `kamn-agent-lib`, and full `kamn-sdk` suites.
- T5 (Verify): ran `cargo fmt --check` and strict clippy for `kamn-agent-lib` and `kamn-sdk`.
- T6 (Process): updated lifecycle docs to Implemented and prepared PR/issue AC evidence.
