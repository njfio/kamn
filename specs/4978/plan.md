# Issue #4978 Plan

- Issue: #4978
- Status: Implemented

## Approach (Implemented)
1. Added a dedicated threshold-ratchet checker that compares current threshold files to a baseline and rejects upward movement.
2. Added waiver parsing/validation with strict schema and mitigation issue linkage requirements.
3. Added deterministic shell contract tests that cover pass/fail/exception/order/parse paths.
4. Integrated the checker into `ci-fast-gate` and artifact publishing.
5. Updated existing CI command-surface and wiring contract tests to include the new checker.

## Affected Modules
- `scripts/ci/check_shell_surface_threshold_ratchet.py`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh`
- `scripts/ci/test_check_shell_surface_threshold_ratchet.sh`
- `.ci/shell-surface-threshold-ratchet-exception.json`
- `.github/workflows/ci-fast-gate.yml`
- `scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/test_ci_tools_command_surface_contract.sh`

## Risks and Mitigations
- Risk level: high
- Mitigation:
  - Fail closed on malformed threshold files, invalid threshold ordering, invalid exception schema, and missing required args.
  - Emit deterministic reason taxonomy markers for every decision path.
  - Keep regression coverage in dedicated script tests plus fast CI tools suite.

## Interface Contract
- No protocol/wire-format changes without explicit approval.
- Checker emits stable markers:
  - `status`, `final_decision`, `reason_taxonomy_version`, `reason_codes`, `reason_codes_csv`
  - `threshold_ratchet_status`, `threshold_ratchet_violations`, `threshold_ratchet_mitigation_issue`
  - `review_required`, `base_commit`
- JSON output schema: `kamn.ci.shell-surface-threshold-ratchet-report.v1`.

## ADR
- No ADR required: no dependency/protocol/architecture boundary change.
