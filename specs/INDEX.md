# Specs Index

specs_index_version=kamn.docs.specs-index.v1
specs_index_purpose=spec navigation and workflow orientation
specs_index_naming_pattern=specs/{issue}-{slug}.md
specs_index_status_taxonomy_csv=planned,active,completed,superseded
specs_index_curated_tracks_csv=m10_phase6_extraction,cli_contract_followups,security_runtime_hardening

## Purpose

Use this index to quickly find current spec tracks and maintain consistent workflow markers.

## Naming Convention

- Pattern: `specs/{issue}-{slug}.md`
- Example: `specs/6417-specs-index-docs-contract.md`

## Status Taxonomy

- `planned`: issue/spec drafted, implementation not started
- `active`: red/green/refactor/integration in progress
- `completed`: merged and closed with integration evidence
- `superseded`: replaced by a newer issue/spec track

## Curated Tracks

### m10_phase6_extraction

- [6407-m10-phase6-runtime-clock-validator-extraction.md](./6407-m10-phase6-runtime-clock-validator-extraction.md) (`completed`)
- [6409-m10-phase6-scheduler-cycle-report-extraction.md](./6409-m10-phase6-scheduler-cycle-report-extraction.md) (`completed`)
- [6411-m10-phase6-budget-overflow-projector-extraction.md](./6411-m10-phase6-budget-overflow-projector-extraction.md) (`completed`)

### cli_contract_followups

- [6413-align-command-activation-root-contract-with-harness-pattern.md](./6413-align-command-activation-root-contract-with-harness-pattern.md) (`completed`)
- [6415-content-bridge-contract-cases-factoring.md](./6415-content-bridge-contract-cases-factoring.md) (`completed`)

### security_runtime_hardening

- [6380-fail-fast-tls-production-paths.md](./6380-fail-fast-tls-production-paths.md) (`completed`)
- [6381-document-service-secret-rotation-runbook.md](./6381-document-service-secret-rotation-runbook.md) (`completed`)
- [6382-cargo-audit-policy-gate-runner-impact.md](./6382-cargo-audit-policy-gate-runner-impact.md) (`completed`)
