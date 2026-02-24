# Plan: Issue #5885

## Approach
1. Baseline shell LOC via hard-ceiling metric script.
2. Select high-blank-line scripts from CI/runtime/kolme surfaces.
3. Apply whitespace-only compaction in multiple implementation commits (one surface/file group per commit).
4. Run targeted regression lanes after each substantial file-group edit; run full verification set before PR.
5. Record post-change metrics and prepare DoD shell-surface markers.

## Affected Modules
- `scripts/ci/test_workflow_scope_policy.sh`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh`
- `scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh`
- `scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh`
- `scripts/runtime/validate_service_api_axum_ingress_live.sh`
- `scripts/runtime/test_check_sqlite_crash_recovery_live_policy.sh`

## Risks / Mitigations
- Risk: accidental logic drift in large shell scripts.
  - Mitigation: whitespace-only edits, run corresponding regression lanes.
- Risk: incomplete validation coverage.
  - Mitigation: include CI policy lane + runtime/kolme policy lanes + review docs contract lane.

## Interfaces / Contracts
- No API/wire/schema contract changes.
- Shell-surface metrics contract remains authoritative via `check_shell_loc_hard_ceiling.sh`.
