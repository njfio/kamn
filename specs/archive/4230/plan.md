# Plan — #4230

Status: Reviewed

## Approach

- Implement `check-evidence-convergence` in `service_api_axum_ingress_live_contract.py`.
- Add deterministic evidence taxonomy and promotion mapping constants.
- Add standalone wrapper script and integrate checker into axum lane wrapper output markers.
- Update release checklist and docs tests to pin reason-mapping references.

## Affected Areas

- `scripts/runtime/service_api_axum_ingress_live_contract.py`
- `scripts/runtime/check_service_api_axum_ingress_live_evidence_convergence.sh`
- `scripts/runtime/validate_service_api_axum_ingress_live_contract_lane.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations

- Risk: mapping function drift between policy and convergence checks.
  - Mitigation: reuse `_resolve_protocol_mismatch_reason_code` as single source for expected mapping.
