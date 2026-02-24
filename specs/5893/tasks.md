# Tasks: Issue #5893 - Branch-Diff Freeze Guard for Existing r51+ Review Docs

1. T1 (Conformance/RED): add branch-diff freeze assertion to `review_r53_docs_contract.rs` and run targeted lane expecting initial failure when assertion is incomplete.
2. T2 (Implementation): implement deterministic branch-diff parsing + existing-doc status guard (`A` only).
3. T3 (Conformance/GREEN): run `cargo test -p kamn-core --test review_r53_docs_contract`.
4. T4 (Regression): verify existing freeze policy assertions still pass in same lane.
