# Spec: Issue #6094 - Reduce Wave-Wrapper Shell Script Count (Post-#6089 Consolidation Pass)

- Issue: #6094
- Status: Implemented
- Type: task
- Priority: P1
- Area: devops
- Milestone: `specs/milestones/r67-runtime-hardening-and-surface-reduction/index.md`
- Last Updated: 2026-02-26
- Parent: #6086

## Problem Statement
Issue #6089 reduced shell LOC but did not reduce shell script file count. Story #6086 remains open because acceptance requires measurable shell script-count reduction with command-surface safety.

## Scope
In scope:
- Delete a bounded set of orphaned shell scripts that have zero in-repo references and no dispatcher registry entries.
- Prove no stale references are introduced.
- Report measured shell script-count and shell LOC deltas.

Out of scope:
- Broad shell surface redesign.
- Wrapper-family command-surface changes for scripts referenced by CI/workflow contracts.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Shell script file count decreases by at least 4 files in this pass.
- AC-2: Deleted scripts are orphaned by evidence (zero in-repo path references; no `scripts/lib/exec_registry.json` entries).
- AC-3: Stale-script reference policy remains green after deletion.
- AC-4: Closure reports shell-surface DoD markers with actual deltas.

## Conformance Cases
- C-01 (Conformance, AC-1): Pre/post telemetry shows `shell_script_file_count` decreases from baseline by >=4.
- C-02 (Conformance, AC-2): For each deleted path, `rg --fixed-strings <path> .` yields no references pre-delete beyond self and no registry mapping exists.
- C-03 (Regression, AC-3): `bash scripts/ci/check_stale_script_references.sh` passes post-delete.
- C-04 (Conformance, AC-4): Issue closure comment and PR summary include `shell_loc_delta_actual`, `rust_loc_delta_actual`, `shell_to_rust_ratio_delta_actual`, `shell_surface_ratio_target_status`.

## Success Metrics / Observable Signals
- `shell_script_file_count` decreases while preserving CI policy checks.
- No stale-script reference violations introduced.
