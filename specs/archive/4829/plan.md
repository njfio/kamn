# Plan — Issue #4829

## Approach

1. Add a canonical registry file (`lane_registry.json`) populated from current manifest and wrapper-symlink artifacts.
2. Add generator/checker script that consumes the registry and supports:
   - `--mode check` for repository drift detection
   - `--mode render` for artifact rendering into an output root
3. Add a shell contract test that validates check-mode markers and render-mode artifact output.
4. Wire the new contract test into framework test runner and verify with full CI tools regression.

## Affected Modules

- `scripts/framework/lane_registry.json`
- `scripts/framework/generate_lane_artifacts.py`
- `scripts/framework/test_lane_registry_generation.sh`
- `scripts/framework/test_contract_framework.sh`
- `docs/architecture/lane-registry-generation.md`

## Risks / Mitigations

- Risk: registry drift from repository artifacts.
  Mitigation: generator check-mode compares registry payload against current manifests/symlink wiring.
- Risk: render mode writes invalid wrapper links.
  Mitigation: explicit wrapper metadata (`wrapper_relpath`, `wrapper_name`, `link_target`) with validation.
- Risk: CI cost increase.
  Mitigation: add guard under existing framework suite and validate in existing `test_ci_tools` regression.

## Interfaces / Contracts

- Registry schema version: `kamn.framework.lane-registry.v1`
- Generator markers:
  - `status=ok|fail`
  - `validation_mode=check|render`
  - `manifest_entries=<n>`
  - `wrapper_entries=<n>`
- Check mode fails closed on:
  - missing manifest/wrapper
  - manifest JSON payload mismatch
  - wrapper symlink mismatch

## ADR

No ADR required for this subtask. No dependency/protocol boundary change introduced.
