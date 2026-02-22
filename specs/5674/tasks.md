# Tasks: #5674 Remove Remaining Agent-Lib Stubs via Service/SDK Route Expansion

- [x] T1 (Conformance/Functional): add RED node service API tests for accept/complete/fund/release route contracts (C-01).
- [x] T2 (Regression/Governance): add RED route matrix + scope-policy fixture coverage updates for new routes (C-02).
- [x] T3 (Implementation): implement node route constants/path helpers/payload/auth updates for new routes (AC-1, AC-2).
- [x] T4 (Conformance/Functional): add RED SDK tests for accept/complete/fund/release method route composition and decode (C-03).
- [x] T5 (Implementation): implement SDK typed models + methods and exports for new routes (AC-3).
- [x] T6 (Conformance/Regression): add RED agent-lib tests for former stub operations and implement SDK-backed methods (C-04, AC-4).
- [x] T7 (Regression): run targeted tests for `kamn-node`, `kamn-sdk`, and `kamn-agent-lib` (C-05).
- [x] T8 (Verify): run `cargo fmt --all --check` and targeted clippy for touched crates.
- [x] T9 (Closeout): update PRD operation coverage markers, set spec status Implemented, and post RED/GREEN evidence on issue.
