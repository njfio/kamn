# Issue #5223 Plan

- Issue: #5223
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Approach
1. Snapshot non-data-layer DID-string callsites and group them into bounded migration waves.
2. Create follow-up implementation subtasks (one per wave) and capture their issue IDs.
3. Add deterministic typed-DID migration markers to:
   - `docs/planning/kamn-data-layer-prd.docx.md`
   - `docs/review/gaps-and-issues-r43.md`
4. Add/extend docs-contract tests to enforce:
   - marker presence
   - numeric field parseability
   - `#<id>` issue-link format for wave plan markers
5. Run targeted tests and shell-ratio guardrail checks.

## Inventory Snapshot Scope (Non-Data-Layer)
- Bridge + marketplace family:
  - `bridge_adapter.rs`, `cross_chain_bridge.rs`, `discord_bridge.rs`, `telegram_bridge.rs`, `service_marketplace.rs`
- Operator + governance family:
  - `operator_binding.rs`, `operator_actions.rs`, `operator_dashboard_api.rs`, `operator_dashboard_ui.rs`, `governance_workflow.rs`, `task_payment.rs`
- Runtime/proof/reputation family:
  - `runtime_peer_coordination.rs`, `runtime_phase_coordination.rs`, `group_channel_crypto.rs`, `message_proof_anchoring.rs`, `reputation_signals.rs`, `reputation_state.rs`, `instruction_verify.rs`, `agent_upgrade_workflow.rs`, `upgrade_orchestration.rs`

## Risks and Mitigations
- Risk: marker drift across planning/review docs.
  - Mitigation: contract tests parse markers and fail closed.
- Risk: ambiguous migration scope.
  - Mitigation: explicit family/module list + wave issue IDs.
- Risk: shell governance regression for planning-only task.
  - Mitigation: Rust/docs-only changes; no shell file edits.

## Interfaces / Contracts
- Marker schema version:
  - `typed_did_migration_inventory_schema_version=kamn.typed-did-migration.inventory.v1`
- Wave issue linkage:
  - marker values must use `#<id>` format and include all planned wave issue IDs.
