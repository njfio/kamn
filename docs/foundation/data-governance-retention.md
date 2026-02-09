# Data Governance Retention and Redaction Contracts

This document captures the replay-safe evidence contract for channel retention
and redaction checks used by fast CI lanes.

## Contract Scope
- Deterministic retention candidate evaluation and prune eligibility.
- Deterministic redaction application evidence with replay-safe markers.
- Schema-validated evidence bundle and fail-closed policy checker behavior.

## Evidence Schema
- `schema_version`: `kamn.channel.retention-redaction-evidence.v1`
- `evidence_key`: `channel_retention_redaction:<lane>:v1`
- `reason_key`: `channel_retention_redaction_reason:<final_decision>:v1`
- `final_decision`: `GO|NO-GO`

## Local Validation Commands
Run from repository root:

```bash
bash scripts/channel/generate_channel_retention_redaction_evidence_bundle.sh --help
bash scripts/channel/check_channel_retention_redaction_policy.sh --help
bash scripts/channel/run_channel_retention_redaction_contract_lane.sh
bash scripts/channel/test_generate_channel_retention_redaction_evidence_bundle.sh
bash scripts/channel/test_run_channel_retention_redaction_contract_lane.sh
cargo test -p kamn-core --test data_governance_retention_docs
```

## Policy Rules
- Evidence bundle schema/version drift fails closed.
- `combined_reason_codes` must be sorted unique deterministic strings.
- `final_decision` must match derived retention/redaction replay-safe policy.
- `evidence_key` and `reason_key` must match deterministic lane/decision keys.

## Regression Marker
- replay-safe reason-code drift is rejected (`Regression: #930`)
