# R50.15 Milestone Index - Cross-Store Replay Policy Marker API

- Milestone: R50.15 Cross-store replay policy marker API
- Scope: Add deterministic policy marker helper API for cross-store replay consistency and integrate it into the contract lane.
- Tracking issue: #5499

## Deliverables
- Issue #5499 lifecycle artifacts (`spec.md`, `plan.md`, `tasks.md`)
- Additive helper API in `crates/kamn-core/src/cross_store_replay_consistency.rs`
- Contract lane integration update in `crates/kamn-core/src/bin/cross_store_replay_consistency_contract_lane.rs`
- Unit tests for helper behavior
