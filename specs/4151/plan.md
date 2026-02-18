# Issue #4151 Plan

- Issue: #4151
- Status: Implemented

## Approach
1. Extend deployment preflight policy checker output with deterministic marker-contract and runbook-parity fields.
2. Add fail-closed marker/schema mismatch classification in checker evaluation.
3. Update shell contract lane assertions to validate new fields in GO and mismatch NO-GO paths.
4. Synchronize runbook markers in deploy/planning docs and enforce with Rust docs-contract tests.

## Affected Modules
- `scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py`
- `scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh`
- `docs/deploy/kolme_devnet_ops.md`
- `docs/planning/kolme-devnet-ops.md`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
- `specs/4151/spec.md`
- `specs/4151/plan.md`
- `specs/4151/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep reason-taxonomy constants explicit and deterministic.
  - Restrict scope to checker/docs/tests to avoid runtime behavior drift.
  - Validate both GO and mismatch NO-GO fixture paths in shell contract lane.

## Interface Contract
- Additive checker output fields only.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this scoped subtask.

## Verification Summary
- `bash scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh` (pass)
- `cargo test -p kamn-core --test kolme_devnet_ops_docs` (pass)
- `cargo test -p kamn-core --test release_gonogo_checklist_docs` (pass)
