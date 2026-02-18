# Plan — Issue #4198

## Approach
- Extend `scripts/runtime/local_full_stack_integration_live_contract.py` `check-policy` path with runbook marker parity verification.
- Introduce deterministic runbook parity markers in policy report/output.
- Wire `scripts/runtime/validate_local_full_stack_integration_live_contract_lane.sh` to pass runbook file into policy checker and surface returned parity markers.
- Replace test-local runbook parity helper in contract-lane shell test with policy/lane-integrated parity assertions.
- Update governance docs and docs-contract tests.

## Affected Modules
- `scripts/runtime/local_full_stack_integration_live_contract.py`
- `scripts/runtime/validate_local_full_stack_integration_live_contract_lane.sh`
- `scripts/runtime/test_check_local_full_stack_integration_live_policy.sh`
- `scripts/runtime/test_validate_local_full_stack_integration_live_contract_lane.sh`
- `docs/deploy/kolme_devnet_ops.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations
- Risk: Existing consumers of `check-policy` may not pass runbook path.
  - Mitigation: default runbook path inside checker; optional override via `--runbook-file`.
- Risk: Marker naming drift between docs and checker output.
  - Mitigation: constants in checker + docs-contract tests for exact marker strings.
- Risk: Added checks cause false negatives in local runs.
  - Mitigation: deterministic precedence for taxonomy vs parity drift and focused tamper regression tests.

## Interfaces/Contracts
- CLI contract update for `check-policy`:
  - Add `--runbook-file` (default `docs/deploy/kolme_devnet_ops.md`).
- Policy output/report additions:
  - `local_full_stack_harness_runbook_marker_parity_status`
  - `local_full_stack_harness_runbook_reason_taxonomy_version`
  - `local_full_stack_harness_runbook_reason_codes_csv`
  - `local_full_stack_harness_runbook_reason_code`

## ADR
- Not required (no dependency/protocol/schema break beyond local script contract markers).
