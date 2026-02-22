# Tasks: #5624 Evidence Contract Status Integration

- T1 (RED): add failing command-contract tests for `evidence_contract` presence, status wiring, and deterministic fail path.
- T2 (Implementation): compute evidence contract markers and wire into `live_execution` aggregation.
- T3 (Regression): verify runtime marker contracts remain unchanged.
- T4 (Docs): add R53 evidence-contract research artifact + docs contract test; update milestone index progress markers.
- T5 (Verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, and `cargo test -p kamn-e2e-harness`.
