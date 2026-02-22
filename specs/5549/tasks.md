# Issue #5549 Tasks - Review Artifact Snapshot Semantics and Reconciliation-Loop Guardrails

## Ordered Tasks
1. T1 (tests/red): add failing R50+ snapshot-semantics docs-contract test.
2. T2 (docs/green): update `docs/review/README.md` with R50+ snapshot marker policy and invariants.
3. T3 (docs/green): add required snapshot markers to `docs/review/gaps-and-issues-r50.md`.
4. T4 (tests/green): relax R49 branch-marker enforcement to informational-only in `review_r49_docs_contract.rs`.
5. T5 (verify): run targeted docs-contract tests and quality gates (`fmt`, `clippy` scoped).
6. T6 (docs/spec): set spec status to Implemented and close issue artifacts.

## Test Tier Mapping
- Unit: marker parser/value parsing in docs-contract tests
- Functional: marker presence contracts in README and review artifact files
- Conformance: C-01..C-05 mapped to dedicated review docs-contract tests
- Integration: R50+ file scanning across `docs/review/gaps-and-issues-r*.md`
- Regression: R49 docs-contract lane kept green while branch markers become informational-only
- Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A with PR justification
