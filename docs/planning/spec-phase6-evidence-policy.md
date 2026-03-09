# Spec Phase 6 Evidence Policy

This document defines the governance contract that blocks closure-ready top-level issue specs from closing without explicit, canonical Phase 6 integration evidence.

## Policy Markers

- `spec_phase6_policy_version=kamn.spec-phase6-evidence-policy.v2`
- `spec_phase6_scope=specs/*.md closure-ready specs`
- `spec_phase6_required_status_gate=Status: Implemented`
- `spec_phase6_canonical_section=## Phase 6 integration evidence`
- `spec_phase6_noncanonical_headings_fail_closed=true`
- `spec_phase6_required_execution_marker=Executed:`
- `spec_phase6_migration_plan_status=defined`
- `spec_phase6_policy_status=verified|fail-closed`

## Enforcement Contract

1. A top-level issue spec (`specs/*.md`) is closure-ready when it contains `Status: Implemented`.
2. Closure-ready specs must use the canonical section heading `## Phase 6 integration evidence`.
3. Legacy headings such as `## Integration Evidence`, `## Phase 6 Evidence`, and `## Phase 6 notes` fail closed for closure-ready specs.
4. Phase 6 evidence must include an `Executed:` marker and at least one concrete executed command in backticks.
5. CI enforcement is fail-closed through `scripts/ci/check_spec_phase6_evidence_policy.sh`.

## Migration Plan

1. New or updated closure-ready specs must normalize to the canonical heading before merge.
2. Historical top-level specs that are not yet closure-ready may keep legacy headings temporarily until they are next updated.
3. Historical top-level specs missing Phase 6 evidence entirely should be backfilled in follow-up issues instead of being rewritten opportunistically.
4. Nested planning archives under `specs/**` are not part of this closure-ready policy contract.

## Remediation

1. Rename any legacy Phase 6 heading to `## Phase 6 integration evidence`.
2. Include `Executed:` and list the concrete verification commands that were run.
3. Re-run the checker and verify `status=ok` and `final_decision=GO` before issue closure.
