# Plan — #4228 Implementation

Status: Implemented

## Approach
- Extend service-api axum validation summary marker set and serialized JSON/stdout outputs.
- Extend policy checker required/verified field enforcement and mismatch taxonomy mappings.
- Extend contract-lane required marker sets, runbook parity markers, and summary propagation fields.
- Update docs and docs-contract tests to match the canonical marker set.

## Contract Additions
- Admission decision taxonomy markers (`accept`, `defer`, `reject`).
- Admission decision runbook parity markers and deterministic drift reason names.

## Mitigations
- Keep existing protocol mismatch and admission budget markers unchanged to avoid cross-issue regression.
