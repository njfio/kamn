# Plan: #5706 Resolve R50 Spec-Volume Non-Regression Cap Breach Blocking Workspace Gate

## Approach
1. Capture RED baseline for
   `review_r50_spec_volume_remediation_docs_contract`.
2. Measure current baseline values:
   - top-level `specs/` directory count
   - exported `kamn-core` module count
   - current spec-to-module ratio
3. Update R50 non-regression markers in
   `docs/review/gaps-and-issues-r50.md`.
4. Update hardcoded marker expectations in
   `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`.
5. Re-run targeted contracts and workspace gate command.

## Affected Modules
- `docs/review/gaps-and-issues-r50.md`
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`
- `specs/5706/{spec.md,plan.md,tasks.md}`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: refreshed baseline immediately drifts on subsequent spec additions.
  Mitigation: keep ratchet checks fail-closed and route future drift through
  explicit issue/spec updates.

- Risk: ratio bound precision mismatch causes flaky comparisons.
  Mitigation: use deterministic rounded marker value and retain tolerance
  semantics already in integration checks.

## Interfaces / Contracts
- R50 non-regression marker contract remains:
  - `baseline_spec_dirs == spec_dir_max`
  - `current_spec_dirs <= spec_dir_max`
  - `current_spec_to_module_ratio <= ratio_max`
- Test suite remains fail-closed for marker presence and numeric coherence.

## ADR
- Not required: no dependency or architecture change.
