# Lane Registry Generation Contract

Issue chain: `#4816` -> `#4829` -> `#4830`.

## Purpose

`scripts/framework/lane_registry.json` is the source-of-truth artifact for:
- framework manifest payloads under `scripts/framework/manifests/*.json`
- wrapper symlink wiring for manifest-backed lanes

Decision record: `docs/architecture/adr-lane-registry-source-of-truth.md`

The generator `scripts/framework/generate_lane_artifacts.py` supports:
- `--mode check`: validate repository artifacts against registry entries
- `--mode render`: render manifests/symlinks into an output root

## Registry Schema

- `schema_version`: `kamn.framework.lane-registry.v1`
- `manifests[]`:
  - `manifest_relpath`
  - `manifest_payload`
- `wrappers[]`:
  - `wrapper_name`
  - `wrapper_relpath`
  - `link_target`

## Validation Contract

`bash scripts/framework/test_lane_registry_generation.sh` enforces:
- registry file presence
- generator check-mode pass over repository artifacts
- render-mode generation into an isolated temp root
- representative manifest + wrapper symlink materialization

This guard is also included in `bash scripts/framework/test_contract_framework.sh`.

## Static Maintenance Retirement

Direct manual editing of `scripts/framework/manifests/*.json` or manifest-backed wrapper symlink wiring is no longer the maintenance path.

Use registry-driven validation/generation:
- `bash scripts/framework/check_lane_registry_drift.sh`
- `python3 scripts/framework/generate_lane_artifacts.py --registry-file scripts/framework/lane_registry.json --repo-root . --mode render --output-root <dir>`

`check_lane_registry_drift.sh` is fail-closed and emits deterministic reason taxonomy markers:
- `kamn.framework.lane-registry-drift-reason-taxonomy.v1`
