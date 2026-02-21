# Issue #5499 Plan - Policy Marker Helper Implementation

## Approach
1. Add `policy_status_marker()` on `CrossStoreReplayConsistencyStatus` and `CrossStoreReplayConsistencyReport`.
2. Add tests validating status/report mappings for both consistent and divergent outcomes.
3. Update contract lane binary to assert/print helper-derived marker.
4. Run targeted `kamn-core` tests and format/lint checks as needed.

## Affected Modules
- `crates/kamn-core/src/cross_store_replay_consistency.rs`
- `crates/kamn-core/src/bin/cross_store_replay_consistency_contract_lane.rs`
- `specs/milestones/r50-15-cross-store-replay-policy-marker-api/index.md`
- `specs/5499/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: behavior drift if helper mapping disagrees with existing status checks.
  - Mitigation: tests cover both statuses and contract lane uses helper output.

## Interfaces / Contracts
- Additive public API in `kamn-core`.

## Validation Strategy
- `cargo test -p kamn-core cross_store_replay`
- `cargo test -p kamn-core --bin cross_store_replay_consistency_contract_lane`
- `cargo fmt --check`
