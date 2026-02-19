# Plan — #4222 Admission Decision Taxonomy + Runbook Marker Parity

Status: Implemented

## Approach
1. Add red tests first for admission decision taxonomy marker assertions and tamper paths in:
   - `scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh`
   - `scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh`
2. Emit admission decision taxonomy markers from validation summary in:
   - `scripts/runtime/validate_service_api_axum_ingress_live.sh`
3. Enforce policy fail-closed checks and deterministic mismatch reasons in:
   - `scripts/runtime/service_api_axum_ingress_live_contract.py`
4. Wire lane required markers and runbook parity requirements in:
   - `scripts/runtime/validate_service_api_axum_ingress_live_contract_lane.sh`
5. Update docs and docs-contract tests:
   - `docs/ci/strategy.md`
   - `docs/foundation/release-gonogo-checklist.md`
   - `docs/deploy/kolme_devnet_ops.md`
   - `docs/ops/configuration.md` (if marker surface is documented there)
   - Rust docs tests under `crates/kamn-core/tests/*`
6. Run targeted verification commands for shell scripts and docs-contract tests.

## Affected Modules
- Runtime shell contracts: `scripts/runtime/*service_api_axum_ingress*`
- Runtime policy checker: `scripts/runtime/service_api_axum_ingress_live_contract.py`
- Operator/CI docs: runbook, strategy, release checklist, ops configuration.
- Rust docs-contract tests in `crates/kamn-core/tests`.

## Risks and Mitigations
- Risk: marker-name drift across script/docs/test surfaces.
  - Mitigation: define one canonical marker set and reuse exact literals in all assertions.
- Risk: reason-code taxonomy changes breaking existing gate expectations.
  - Mitigation: add new admission-decision-specific markers without removing existing protocol mismatch taxonomy markers.
- Risk: unintended CI cost increase.
  - Mitigation: keep contract-lane command surface unchanged; only marker validation is extended.

## Interfaces / Contracts
New summary/policy markers:
- `admission_decision_taxonomy_status=verified`
- `admission_decision_accept_status=verified`
- `admission_decision_defer_status=verified`
- `admission_decision_reject_status=verified`
- `admission_decision_reason_taxonomy_version=kamn.runtime.service-api-admission-decision-reason-taxonomy.v1`
- `admission_decision_reason_codes_csv=admission_decision_accept,admission_decision_defer,admission_decision_reject`

Runbook parity markers:
- `admission_decision_taxonomy_mapping_status=verified`
- `admission_decision_runbook_marker_parity_status=verified`
- `admission_decision_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.service-api-axum-admission-decision-runbook-reason-taxonomy.v1`
- `admission_decision_taxonomy_runbook_reason_codes_csv=admission_decision_taxonomy_mapping_drift_detected,admission_runbook_marker_parity_mismatch`

No dependency changes.
