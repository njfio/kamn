# Tasks: #5678 Rebaseline R50 Spec-Volume Non-Regression Ratchet Markers

- [x] T1 (Conformance/Regression): run RED docs-contract test and capture failing assertion (C-03).
- [x] T2 (Implementation): update R50 non-regression marker values to a fresh baseline (C-01).
- [x] T3 (Implementation): update docs-contract test hard-coded expectation values (C-02).
- [x] T4 (Regression): re-run the failing test target to GREEN (C-03).
- [x] T5 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-core -- -D warnings`.
- [x] T6 (Closeout): update issue process log with RED/GREEN evidence.
