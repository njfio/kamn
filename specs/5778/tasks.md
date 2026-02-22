# Tasks: #5778 Reconcile R53 Portable-Agent Stalled Markers After Query-Surface Delivery

- [x] T1 (Conformance/RED): add/adjust `review_r53_docs_contract` assertions for new portable-agent post-publication markers; run lane and capture failure.
- [x] T2 (Implementation): update `docs/review/gaps-and-issues-r53.md` with post-publication reconciliation marker block preserving snapshot semantics.
- [x] T3 (Implementation): perform compensating archive cleanup and update `specs/archive/index.md`.
- [x] T4 (Conformance/GREEN): run `cargo test -p kamn-core --test review_r53_docs_contract` and archive-policy checker.
- [x] T5 (Verify): run fmt/clippy and workspace gate.
- [x] T6 (Closure): set spec status Implemented, update milestone index, close issue.
