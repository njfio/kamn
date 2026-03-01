# Issue 6261 Spec

Status: Reviewed
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6256

## Problem Statement
Shell-surface governance identified risky `eval` usage in four active scripts and two time-bounded CI waivers approaching expiry. Current shell metrics remain below hard ceiling but still require explicit waiver handling for script-surface budget overage.

## Scope
In scope:
- Remove `eval` usage from the four identified scripts.
- Keep command execution behavior equivalent (dry-run/run semantics preserved).
- Refresh expiring waiver metadata with explicit tracked mitigation linkage.
- Re-verify shell-rust ratio, shell hard ceiling, and script-surface budget checks.

Out of scope:
- Large-scale script-to-Rust migration wave.
- CI workflow topology changes.

## Acceptance Criteria
- AC-1: `rg -n "\\beval\\b" scripts .github/workflows` returns no active `eval` usages in production scripts.
- AC-2: Affected script tests/contract checks remain green after `eval` removal.
- AC-3: `.ci/script-surface-budget-waiver.json` and `.ci/fast-gate-budget-delta-waiver.json` are updated with valid future expiry and mitigation linkage for this remediation wave.
- AC-4: Shell governance checks pass:
  - shell-rust ratio guardrail
  - shell LOC hard ceiling
  - script duplication/surface budget (with explicit waiver state)

## Conformance Cases
- C-01 (AC-1): `rg -n "\\beval\\b" scripts .github/workflows` emits zero lines.
- C-02 (AC-2): targeted script contract tests pass for touched scripts.
- C-03 (AC-3): waiver JSON files parse and include non-expired `expires_on` values and mitigation linkage.
- C-04 (AC-4):
  - `bash scripts/ci/check_shell_rust_ratio_guardrail.sh ...` returns `final_decision=GO`.
  - `bash scripts/ci/check_shell_loc_hard_ceiling.sh ...` returns `final_decision=GO`.
  - `bash scripts/ci/check_script_duplication_budget.sh ...` returns `status=pass` with explicit waiver reporting where applicable.

## Test Mapping
- Functional: eval removal behavior and shell governance checks.
- Conformance: C-01..C-04 command outputs.
- Regression: existing script contract tests for touched scripts.
- Unit/Property/Fuzz/Mutation/Performance: N/A for this shell-governance maintenance task.
