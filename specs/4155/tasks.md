# Issue #4155 Tasks

## T1 Red/Green Tests
- [x] Extend rollback-lineage-missing regression assertions to verify deterministic upgrade-lineage and promotion-gate reason mapping markers.
- [x] Extend recovery-lineage-missing regression assertions to verify deterministic upgrade-lineage and promotion-gate reason mapping markers.

## T2 Documentation
- [x] Update `docs/foundation/release-gonogo-checklist.md` with explicit rollback/recovery lineage fail-closed promotion-gate mapping markers.

## T3 Verification
- [x] Run `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`.
- [x] Run `bash scripts/ci/check_stale_script_references.sh --output-json /tmp/stale-script-reference-report.json`.
- [x] Run `cargo fmt --check`.
