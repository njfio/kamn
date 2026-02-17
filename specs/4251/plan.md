# Plan — #4251 Partition-Healing Deterministic Reconciliation Contracts

Status: Implemented

## Approach

1. Extend `block_reconciliation_partition_rejoin_live_contract.py` policy output with deterministic mismatch-reason mapping markers.
2. Add mapping resolver that projects a stable reason category from policy failed checks.
3. Expand policy checker tests for missing marker and nondeterministic reason-code payload failures.
4. Update contract lane marker checks and docs parity surfaces.
5. Add/extend docs contract tests for release checklist marker coverage.

## Affected Modules

- `scripts/runtime/block_reconciliation_partition_rejoin_live_contract.py`
- `scripts/runtime/test_check_block_reconciliation_partition_rejoin_live_policy.sh`
- `scripts/runtime/validate_block_reconciliation_partition_rejoin_live_contract_lane.sh`
- `scripts/runtime/test_validate_block_reconciliation_partition_rejoin_live_contract_lane.sh`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks + Mitigations

- Risk: changing policy output markers may break existing lane consumers.
  - Mitigation: keep existing markers unchanged and only add deterministic mapping fields.
- Risk: brittle docs marker assertions.
  - Mitigation: assert deterministic marker lines only.

## Interfaces / Contracts

- New policy output markers:
  - `partition_healing_mismatch_reason_mapping_status`
  - `partition_healing_mismatch_reason_taxonomy_version`
  - `partition_healing_mismatch_reason_codes_csv`
  - `partition_healing_mismatch_reason_code`

## ADR

Not required (no dependency/protocol format/architecture change).
