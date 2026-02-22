# Issue #5555 Tasks - R50 Governance-Feature Activity Non-Regression Ratchet Enforcement

## Ordered Tasks
1. T1 (tests/red): extend `review_r50_governance_feature_rebalancing_docs_contract.rs` with failing non-regression ratchet assertions.
2. T2 (docs/green): add non-regression ratchet markers to `docs/review/gaps-and-issues-r50.md`.
3. T3 (docs/green): add ratchet schema/invariants to `docs/review/README.md`.
4. T4 (tests/green): rerun targeted review docs-contract lanes and keep existing lanes green.
5. T5 (closeout): mark spec Implemented and complete issue/PR closure artifacts.

## Test Tier Mapping
- Unit: marker parse/ratio parse assertions
- Functional: marker presence checks in README and R50 review artifact
- Conformance: C-01..C-05 via docs-contract lanes
- Integration: ratio bound checks against current activity-ratio markers
- Regression: existing activity-ratio/review lanes remain green
- Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A with PR justification
