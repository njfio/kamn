# Plan — Issue #4820

## Approach

- Add a migration-wave conformance test that fails until all selected scripts source `common.sh` and remove duplicate helper blocks.
- Apply a mechanical, bounded migration pass over selected `test_generate_*evidence*` scripts.
- Run the migrated target suite end-to-end to confirm no behavior drift.

## Affected Modules

- `scripts/framework/test_common_shell_migration_wave_evidence_bundle.sh`
- selected evidence-bundle tests across:
  - `scripts/bridge/`
  - `scripts/canary/`
  - `scripts/channel/`
  - `scripts/compliance/`
  - `scripts/cutover/`
  - `scripts/did/`
  - `scripts/escrow/`
  - `scripts/governance/`
  - `scripts/kolme/`
  - `scripts/message/`
  - `scripts/reputation/`
  - `scripts/runtime/`
  - `scripts/sdk/`
  - `scripts/signer/`
  - `scripts/task/`
  - `scripts/token/`
  - `scripts/treasury/`

## Risks / Mitigations

- Risk: migration drift or hidden coupling across scripts/wrappers/manifests.
  Mitigation: bounded file list + deterministic migration-wave contract test + full migrated-suite execution.
- Risk: CI cost increase.
  Mitigation: target only scripts with existing fast deterministic test coverage.

## Interfaces / Contracts

- Preserve existing lane entrypoint compatibility unless explicitly versioned.
- Emit stable key=value outputs and reason taxonomy/version markers on policy paths.

## ADR

- Required only if this issue introduces protocol/dependency/architecture decisions.
