# Plan: #4359 Deployment Safety Gate Convergence

## Approach

1. Extend milestone aggregate validation in `scripts/deploy/gonogo_evidence_contract.py` to require:
- deployment preflight policy rotation taxonomy markers from #4358
- go/no-go gate CI/local boundary markers proving low-cost CI smoke plus local-heavy drill separation
2. Add RED fixtures/assertions in `scripts/deploy/test_generate_gonogo_evidence_bundle.sh` for:
- rotation taxonomy drift
- boundary marker drift
3. Keep deterministic fail-closed behavior by mapping drift to stable reason codes and preserving sorted reason-code output.
4. Update docs in `docs/ci/strategy.md` with the new deployment safety convergence markers.

## Affected Modules

- `scripts/deploy/gonogo_evidence_contract.py`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/deploy/gonogo_evidence_contract_lane_contract.sh` (fixture parity updates if required)
- `docs/ci/strategy.md`

## Risks and Mitigations

- Risk: fixture drift due added required marker fields.
- Mitigation: update deterministic fixture payloads in tests and assert fail-closed mismatch reasons.

- Risk: reason-code taxonomy expansion drifts from docs contracts.
- Mitigation: update docs marker section and run docs contract tests.

## Interfaces / Contracts

- Milestone review bundle in `kamn.release.milestone-review-bundle.v1` remains schema-stable but tightens required observed/contracts fields.
- Deterministic reason-code set extends for rotation taxonomy and boundary mismatch classes.

## ADR

No ADR required (no dependency/protocol architecture change).
