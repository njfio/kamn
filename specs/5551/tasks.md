# Issue #5551 Tasks - R50 Spec-Volume Non-Regression Ratchet Guardrail Enforcement

## Ordered Tasks
1. T1 (tests/red): extend `review_r50_spec_volume_remediation_docs_contract.rs` with failing ratchet marker + current-count assertions.
2. T2 (docs/green): add non-regression ratchet markers to `docs/review/gaps-and-issues-r50.md`.
3. T3 (docs/green): add ratchet schema/invariants to `docs/review/README.md`.
4. T4 (tests/green): rerun targeted docs-contract suites and verify existing review contracts remain green.
5. T5 (closeout): set spec status Implemented and complete issue/PR closure artifacts.

## Test Tier Mapping
- Unit: marker parser and numeric conversion paths in docs-contract test
- Functional: marker presence checks in README and R50 artifact
- Conformance: C-01..C-05 via review docs-contract assertions
- Integration: current repository count computation versus declared ratchet maxima
- Regression: existing release review marker lanes remain green
- Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A with PR justification
