# Plan: Issue #6094

## Approach
1. Establish baseline shell-script count/LOC telemetry (`scripts/**/*.sh`).
2. Remove only orphaned shell scripts that satisfy all of:
   - zero in-repo fixed-string path references,
   - zero `scripts/lib/exec_registry.json` mappings,
   - no workflow/contract fixture references.
3. Recompute post-change telemetry and run stale-reference guard.
4. Publish measured shell-surface deltas in PR + issue closure.

## Candidate Deletion Set (validated in-analysis)
- `scripts/ci/check_markdown.sh`
- `scripts/message/test_run_processor_proof_artifact_contract_lane.sh`
- `scripts/runtime/test_validate_daemon_os_signal_live.sh`
- `scripts/signer/test_run_signer_policy_contract_lane.sh`

## Affected Modules/Areas
- `scripts/ci/`
- `scripts/message/`
- `scripts/runtime/`
- `scripts/signer/`
- `specs/6094/`

## Risks and Mitigations
- Risk: hidden dynamic invocation not captured by static references.
  Mitigation: restrict to orphan scripts with no registry mapping and run stale-reference policy checks.
- Risk: accidental contract-surface drift.
  Mitigation: avoid deleting any script with known wrapper matrix / manifest references.

## Interfaces / Contracts
- No runtime API/trait/wire changes.
- CI contract touched: stale-script reference policy (`scripts/ci/check_stale_script_references.sh`).

## ADR
- Not required (no dependency, protocol, or architecture decision change).
