# Plan — Issue #4200

## Approach
- Extend `scripts/runtime/go_no_go_gate_lane_contract.py` with:
  - convergence-verifier taxonomy constants and reason projection
  - promotion decision reason-mapping taxonomy constants and normalization helpers
  - new report/output markers for convergence status + mapped reason outputs
- Extend `scripts/runtime/test_run_go_no_go_gate_lane.sh` assertions for baseline and tamper mapping markers.
- Update `docs/foundation/release-gonogo-checklist.md` with marker references.
- Extend `crates/kamn-core/tests/release_gonogo_checklist_docs.rs` to enforce docs-marker parity.

## Affected Modules
- `scripts/runtime/go_no_go_gate_lane_contract.py`
- `scripts/runtime/test_run_go_no_go_gate_lane.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations
- Risk: Reason-mapping changes alter existing downstream expectations.
  - Mitigation: additive markers; retain existing `reason_codes` fields unchanged.
- Risk: Category mapping misses future reason forms.
  - Mitigation: explicit fallback classification marker and deterministic ordered normalization.

## Interfaces/Contracts
- No CLI argument changes.
- Add report/output markers:
  - `promotion_evidence_convergence_status`
  - `promotion_evidence_reason_taxonomy_version`
  - `promotion_evidence_reason_codes_csv`
  - `promotion_evidence_reason_code`
  - `promotion_decision_reason_mapping_status`
  - `promotion_decision_reason_taxonomy_version`
  - `promotion_decision_reason_codes_csv`
  - `promotion_decision_reason_code`

## ADR
- Not required (script-level contract extension without dependency/protocol migration).
