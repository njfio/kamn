# Plan — Issue #4819

## Approach

- Add shared shell primitives to `scripts/lib/common.sh` and keep behavior deterministic.
- Migrate a bounded pilot set (three deploy tests + non-Kolme dispatcher) to source `common.sh`.
- Preserve contract output compatibility by reusing existing fallback taxonomy/reason-code variables.
- Validate with focused regression scripts before widening migration scope.

## Affected Modules

- `scripts/lib/common.sh`
- `scripts/framework/test_common_shell_library.sh`
- `scripts/deploy/test_generate_dr_evidence_bundle.sh`
- `scripts/deploy/test_generate_staging_rehearsal_bundle.sh`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/framework/run_non_kolme_contract_lane_dispatch.sh`

## Risks / Mitigations

- Risk: migration drift or hidden coupling across scripts/wrappers/manifests.
  Mitigation: phased rollout with deterministic regression suites and compatibility checks.
- Risk: CI cost increase.
  Mitigation: keep pilot verification focused on touched scripts and existing dispatcher matrix tests.
- Risk: fallback output regression in dispatcher failures.
  Mitigation: preserve `FALLBACK_REASON_TAXONOMY_VERSION` and `FALLBACK_REASON_CODES_CSV` variable override semantics in shared helper.

## Interfaces / Contracts

- Preserve existing lane entrypoint compatibility unless explicitly versioned.
- Emit stable key=value outputs and reason taxonomy/version markers on policy paths.

## ADR

- Required only if this issue introduces protocol/dependency/architecture decisions.
