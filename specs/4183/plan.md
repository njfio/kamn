# Plan — Issue #4183

## Approach

1. Extend the compatibility contract lane checker orchestration to validate runbook parity markers
   against deterministic checker outputs.
2. Add explicit fail-closed reason codes for runbook command and taxonomy marker divergence.
3. Update required docs marker references and docs-contract assertions.
4. Validate with script lanes + targeted Rust docs tests.

## Affected Modules

- `scripts/kolme/contracts/version_compatibility_contract_lane.py`
- `scripts/kolme/test_run_version_compatibility_contract_lane.sh`
- `docs/deploy/kolme_devnet_ops.md`
- `docs/foundation/release-gonogo-checklist.md`
- Rust docs tests including `include_str!` marker contracts

## Risks / Mitigations

- Risk: adding marker checks in more docs can introduce brittle failures.
  Mitigation: enforce only stable, machine-readable markers and command strings.

## Interfaces / Contracts

- Compatibility contract lane must expose deterministic reason mapping for taxonomy/runbook
  divergence.

## ADR

- Not required (no architectural/protocol change).
