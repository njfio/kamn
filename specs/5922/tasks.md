# Tasks: Issue #5922 - Task: Replace fake SHA-256 labels with real sha2::Sha256 in data layer M0-M5

- Issue: #5922
- Spec: `specs/5922/spec.md`
- Plan: `specs/5922/plan.md`
- Status: Implemented
- Last Updated: 2026-02-24

## Ordered Tasks
- T1 (RED / Conformance): added `data_layer_sha256_contract` failing tests for SHA-256 vector parity and pseudo-digest helper removal.
- T2 (GREEN / Implementation): added shared `data_layer_hashing` helper and rewired M0-M5 digest call sites.
- T3 (Refactor): removed duplicated `deterministic_digest_256_hex` functions from M0-M5.
- T4 (Regression): ran `data_layer_sha256_contract` plus M0-M5 integration suites to verify behavior.
- T5 (Verify): ran `cargo fmt --check` and strict `kamn-core` clippy.
- T6 (Process): updated `spec/plan/tasks` status to Implemented for issue lifecycle closure.
