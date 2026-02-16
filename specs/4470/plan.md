# Plan: Issue #4470

Status: In Progress
Issue: #4470

## Approach

1. Add incident-readiness gate source contract constants to go/no-go evidence contract.
2. Implement gate builder with deterministic reason-code taxonomy and normalized outputs.
3. Implement checker convergence validation for serialized incident gate payloads.
4. Expose deterministic gate markers in generator/checker CLI outputs.
5. Update docs and docs tests for schema/taxonomy drift protection.

## Affected Modules

- `scripts/deploy/gonogo_evidence_contract.py`
- `scripts/deploy/gonogo_evidence_contract_lane_contract.sh`
- `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
- `docs/ops/incident-readiness.md`
- `crates/kamn-core/tests/incident_readiness_docs.rs`

## Risks / Mitigations

- Risk: taxonomy drift between staging rehearsal bundle and go/no-go gate expectations.
  - Mitigation: validate nested schema/version surfaces and signoff markers explicitly.
- Risk: docs may diverge from reason code taxonomy.
  - Mitigation: add docs assertions for full reason-code contract set.
