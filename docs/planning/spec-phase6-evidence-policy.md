# Spec Phase 6 Evidence Policy

This document defines the governance contract that blocks closure-ready specs from closing without explicit Phase 6 integration evidence.

## Policy Markers

- `spec_phase6_policy_version=kamn.spec-phase6-evidence-policy.v1`
- `spec_phase6_scope=specs/*.md closure-ready specs`
- `spec_phase6_required_status_gate=Status: Implemented`
- `spec_phase6_required_section=## Phase 6 integration evidence`
- `spec_phase6_required_execution_marker=Executed:`
- `spec_phase6_policy_status=verified|fail-closed`

## Enforcement Contract

1. A top-level issue spec (`specs/*.md`) is closure-ready when it contains `Status: Implemented`.
2. Closure-ready specs must include the section heading `## Phase 6 integration evidence`.
3. Phase 6 evidence must include an `Executed:` marker and at least one concrete executed command in backticks.
4. CI enforcement is fail-closed through `scripts/ci/check_spec_phase6_evidence_policy.sh`.

## Remediation

1. Add or fix the `## Phase 6 integration evidence` section in the flagged spec.
2. Include `Executed:` and list the concrete verification commands that were run.
3. Re-run the checker and verify `status=ok` and `final_decision=GO` before issue closure.
