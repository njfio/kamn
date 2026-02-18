# Plan — Issue #4879

## Approach

- Start with failing spec-derived checks for target conformance cases.
- Implement the smallest deterministic change that satisfies ACs.
- Preserve fast-gate budget and compatibility contracts while reducing shell-surface duplication where applicable.

## Affected Modules

- `scripts/lib/test_json_write_helper_migration_contract.sh`
- `scripts/ci/evaluate_budget.sh`
- `scripts/ci/generate_performance_smoke_report.sh`
- `scripts/bridge/test_generate_bridge_replay_redaction_evidence_bundle.sh`
- `scripts/bridge/test_generate_localhost_bridge_demo_evidence_bundle.sh`
- `scripts/channel/test_generate_channel_retention_redaction_evidence_bundle.sh`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/message/test_generate_group_sender_replay_ratchet_evidence_bundle.sh`
- `scripts/message/test_generate_key_lifecycle_invariant_evidence_bundle.sh`
- `scripts/reputation/test_generate_weighted_decay_property_evidence_bundle.sh`
- `docs/ops/configuration.md`

## Risks / Mitigations

- Risk: migration drift or hidden coupling across scripts/wrappers/checkers.
  Mitigation: phased rollout with compatibility checks and deterministic regression lanes.
- Risk: CI runtime growth.
  Mitigation: retain bounded fast-gate budgets and enforce explicit threshold checks.

## Interfaces / Contracts

- Preserve existing lane entrypoint compatibility unless explicitly versioned.
- Keep reason taxonomy/version markers deterministic and fail closed on drift.

## ADR

- Required if implementation introduces architecture/dependency/protocol strategy changes.
