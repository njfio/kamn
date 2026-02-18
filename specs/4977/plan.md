# Issue #4977 Plan

- Issue: #4977
- Status: Implemented

## Approach (Implemented)
1. Verify `ci-fast-gate` wiring includes both shell-rust ratio guardrail and shell LOC hard-ceiling checks.
2. Capture deterministic fail-closed evidence for both checkers using temporary failing thresholds.
3. Run scoped checker/wiring command-surface contract tests.
4. Close spec lifecycle artifacts for issue #4977.

## Affected Modules
- `.github/workflows/ci-fast-gate.yml` (verified integration target)
- `scripts/ci/check_shell_rust_ratio_guardrail.sh` (verified checker path)
- `scripts/ci/check_shell_loc_hard_ceiling.sh` (verified checker path)
- `scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh` (verified wiring contract)
- `scripts/ci/test_check_shell_rust_ratio_guardrail.sh`
- `scripts/ci/test_check_shell_loc_hard_ceiling.sh`
- `scripts/ci/test_ci_tools_command_surface_contract.sh`
- `specs/4977/spec.md`
- `specs/4977/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigation:
  - Keep fail-closed checker behavior deterministic and reason-taxonomy backed.
  - Validate workflow wiring via contract tests so future edits cannot silently remove required checks.
  - Keep closure PR limited to issue lifecycle artifacts.

## Interface Contract
- No protocol/wire-format changes without explicit approval.
- Reason taxonomy markers remain stable:
  - `kamn.ci.shell-rust-ratio-guardrail-reason-taxonomy.v1`
  - `kamn.ci.shell-loc-hard-ceiling-reason-taxonomy.v1`

## ADR
- No ADR required; no dependency/protocol/architecture change introduced.
