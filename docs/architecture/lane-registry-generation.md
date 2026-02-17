# Lane Registry Generation Contract

Issue chain: `#4816` -> `#4829` -> `#4830`.

## Purpose

`scripts/framework/lane_registry.json` is the source-of-truth artifact for:
- framework manifest payloads under `scripts/framework/manifests/*.json`
- wrapper symlink wiring for manifest-backed lanes

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
