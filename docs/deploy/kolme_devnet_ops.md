# Kolme Devnet Ops (Compatibility Redirect)

Canonical operations guidance for Kolme local-heavy and combined runtime
validation lives in:

- `docs/planning/kolme-devnet-ops.md`

This compatibility path is kept so issue and PR references using
`docs/deploy/kolme_devnet_ops.md` remain valid.

## Release Evidence and Remediation Matrix

Release go/no-go evidence must include deterministic combined native libp2p +
Kolme markers from `kamn.runtime.go-no-go-gate-report.v1`:

- `combined_reason_taxonomy_version=kamn.runtime.local-full-stack-integration-reason-taxonomy.v1`
- `combined_transport_reason_codes=["fork_choice_stale_block_height"]`
- `combined_kolme_runtime_reason_code in {"not_run","live_runtime_integration_passed"}`
- `kolme_runtime_commit_failure_taxonomy_version=v1`
- `kolme_fixture_profile=real-node-non-synthetic-v1`
- `kolme_fixture_profile_version=v1`
- `combined_lane_marker_contract_status=verified`

Fail-closed remediation reason codes are emitted in milestone bundles by
`scripts/deploy/gonogo_evidence_contract.py` and must block promotion when
present, including:

- `milestone_review_go_no_go_gate_combined_reason_taxonomy_version_mismatch`
- `milestone_review_go_no_go_gate_combined_transport_reason_codes_mismatch`
- `milestone_review_go_no_go_gate_combined_kolme_runtime_reason_code_mismatch`
- `milestone_review_go_no_go_gate_kolme_runtime_commit_failure_taxonomy_version_mismatch`
- `milestone_review_go_no_go_gate_kolme_fixture_profile_mismatch`
- `milestone_review_go_no_go_gate_kolme_fixture_profile_version_mismatch`
- `milestone_review_go_no_go_gate_kolme_fixture_profile_status_mismatch`
- `milestone_review_go_no_go_gate_combined_lane_marker_contract_status_mismatch`
