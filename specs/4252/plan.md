# Plan — #4252 Finality Taxonomy and Runbook Parity

Status: Reviewed

## Approach

1. Extend libp2p process-isolated convergence lane report markers with deterministic finality
   taxonomy/runbook parity fields.
2. Extend convergence policy checker with:
   - runbook-file input and marker parity enforcement,
   - fail-closed deterministic reason mapping for taxonomy/runbook drift,
   - policy report marker projection for runbook parity status and resolved reason code.
3. Add Red/regression tests for taxonomy drift and runbook marker divergence.
4. Update contract-lane checks and docs/tests to keep strategy/runbook/release checklist parity.

## Affected Surfaces

- `scripts/runtime/libp2p_convergence_process_isolated_live_contract.py`
- `scripts/runtime/test_check_libp2p_convergence_process_isolated_live_policy.sh`
- `scripts/runtime/validate_libp2p_convergence_process_isolated_live_contract_lane.sh`
- `scripts/runtime/test_validate_libp2p_convergence_process_isolated_live_contract_lane.sh`
- `scripts/runtime/test_validate_libp2p_convergence_process_isolated_live.sh`
- `docs/deploy/kolme_devnet_ops.md`
- `docs/ci/strategy.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/{kolme_devnet_ops_docs.rs,release_gonogo_checklist_docs.rs,block_pipeline_docs.rs}`

## Risks and Mitigations

- Risk: marker contract drift across checker/lane/docs.
  Mitigation: enforce parity in policy checks and doc-contract tests.
- Risk: brittle runbook assertions.
  Mitigation: assert deterministic marker strings only (no prose-shape coupling beyond required markers).

## Interface and Contract Notes

- Add policy checker CLI argument: `--runbook-file` with deterministic default path.
- Add deterministic parity markers and reason-code mapping:
  - `finality_taxonomy_mapping_status`
  - `runbook_marker_parity_status`
  - `finality_taxonomy_runbook_reason_taxonomy_version`
  - `finality_taxonomy_runbook_reason_codes_csv`
  - `finality_taxonomy_runbook_reason_code`

ADR: Not required (no architecture/protocol/dependency decision change).
