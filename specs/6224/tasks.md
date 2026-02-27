# Issue 6224 Tasks

- T1 (Red/Conformance): Add runtime-guards tests for quota/fairness policy rejection/allow boundaries and reason marker mapping.
- T2 (Green/Conformance): Add core compatibility tests for quota/fairness re-export API and deterministic markers.
- T3 (Regression): Run targeted tests for added modules.
- T4 (Verification): Run `cargo test -p kamn-runtime-guards` and targeted `cargo test -p kamn-core` commands, then map ACs to tests.
