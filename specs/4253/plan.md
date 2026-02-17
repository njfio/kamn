# Plan — #4253 Finality Evidence Convergence Checker

Status: Reviewed

## Approach

1. Extend `libp2p_convergence_process_isolated_live_contract.py` with:
   - deterministic promotion decision reason mapping in policy output,
   - `check-evidence-convergence` subcommand validating lane/policy/source artifact linkage.
2. Add shell wrapper `check_libp2p_convergence_process_isolated_live_evidence_convergence.sh`.
3. Wire convergence checker into
   `validate_libp2p_convergence_process_isolated_live_contract_lane.sh` and emit lane markers.
4. Add red/regression tests for missing-link, tamper, and mapping-drift fail-closed paths.
5. Update planning/checklist/runbook docs and Rust docs-contract tests for marker parity.

## Affected Surfaces

- `scripts/runtime/libp2p_convergence_process_isolated_live_contract.py`
- `scripts/runtime/check_libp2p_convergence_process_isolated_live_evidence_convergence.sh`
- `scripts/runtime/validate_libp2p_convergence_process_isolated_live_contract_lane.sh`
- `scripts/runtime/test_check_libp2p_convergence_process_isolated_live_policy.sh`
- `scripts/runtime/test_validate_libp2p_convergence_process_isolated_live_contract_lane.sh`
- `scripts/runtime/test_check_libp2p_convergence_process_isolated_live_evidence_convergence.sh`
- `docs/planning/kolme-devnet-ops.md`
- `docs/foundation/release-gonogo-checklist.md`
- `docs/deploy/kolme_devnet_ops.md`
- `crates/kamn-core/tests/{kolme_devnet_ops_docs.rs,release_gonogo_checklist_docs.rs,block_pipeline_docs.rs}`

## Risks and Mitigations

- Risk: reason-taxonomy drift between policy and convergence checker.
  Mitigation: centralize reason-code resolver and assert parity in convergence checker tests.
- Risk: brittle docs contract assertions.
  Mitigation: assert only deterministic marker strings and command references.

## Interface and Contract Notes

- New CLI path:
  - `python3 scripts/runtime/libp2p_convergence_process_isolated_live_contract.py check-evidence-convergence --report-file <lane> --policy-file <policy> --output-json <path>`
- New policy markers:
  - `promotion_decision_reason_mapping_status`
  - `promotion_decision_reason_taxonomy_version`
  - `promotion_decision_reason_codes_csv`
  - `promotion_decision_reason_code`
- New convergence markers:
  - `evidence_convergence_status`
  - `reason_taxonomy_version`
  - `reason_codes_csv`
  - `reason_codes_value`

ADR: Not required (no dependency/protocol architecture change).
