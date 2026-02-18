# Plan — Issue #4821

## Approach

- Add a RED assertion in manifest-backed dispatcher matrix tests that requires `wrapper_name`/`phase` metadata.
- Migrate non-Kolme manifests to include `wrapper_name` and `phase`.
- Replace shell case-statement resolution with a metadata-driven resolver helper and fail-closed error mapping.
- Verify with full non-Kolme wrapper matrix coverage and deep-lane dispatch checks.

## Affected Modules

- `scripts/framework/run_non_kolme_contract_lane_dispatch.sh`
- `scripts/framework/resolve_non_kolme_manifest.py`
- `scripts/framework/test_non_kolme_manifest_backed_contract_lane_dispatch_wrapper_matrix.sh`
- `scripts/framework/manifests/*.json` (non-Kolme wrapper-backed manifests)

## Risks / Mitigations

- Risk: migration drift or hidden coupling across scripts/wrappers/manifests.
  Mitigation: metadata validation in matrix tests + full wrapper matrix regression execution.
- Risk: CI cost increase.
  Mitigation: reuse existing fast non-Kolme matrix suites already in CI fast gate.

## Interfaces / Contracts

- Preserve existing lane entrypoint compatibility unless explicitly versioned.
- Emit stable key=value outputs and reason taxonomy/version markers on policy paths.

## ADR

- Required only if this issue introduces protocol/dependency/architecture decisions.
