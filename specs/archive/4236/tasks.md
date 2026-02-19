# Tasks — #4236 Journal Append and Checkpoint Marker Integrity

Status: Reviewed

- T1 (Regression): add red tamper fixtures for WAL append mismatch + append/checkpoint parity mismatch (`#4240`).
- T2 (Implementation): add deterministic append/checkpoint integrity output markers and parity fail-closed reason mapping (`#4241`).
- T3 (Docs): update ops/release/CI marker contracts and Rust docs tests.
- T4 (Verification): run runtime policy/lane tests and docs contract tests.
