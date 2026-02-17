# Plan — #4298

Status: Reviewed

## Approach

- Add RED shell tests for missing, stale, and malformed TLS evidence failures in existing go/no-go evidence bundle tests.
- Implement deterministic normalization for TLS reason marker projection in `gonogo_evidence_contract.py`.
- Ensure generated evidence bundles include TLS taxonomy + reason marker fields consumed by lane contracts.
- Keep smoke path low-cost by reusing existing checker execution paths and fixtures.

## Affected Areas

- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/deploy/gonogo_evidence_contract.py`
- `scripts/deploy/gonogo_evidence_contract_lane_contract.sh`
- `docs/security/tls-hardening.md`
- `docs/foundation/release-gonogo-checklist.md`

## Risks and Mitigations

- Risk: Non-deterministic reason ordering causes flaky tests.
  - Mitigation: normalize/sort reason lists before emitting marker values.
- Risk: Overly strict freshness window creates false negatives.
  - Mitigation: keep explicit max-age contract and fixed fixtures in tests.
- Risk: Scope creep into deploy pipeline internals.
  - Mitigation: constrain changes to checker output and lane contract markers.

## Interfaces and Contracts

- TLS evidence marker outputs: taxonomy version, reason codes CSV/value, final decision.
- Deterministic fail-closed reason mapping for missing/stale/invalid evidence conditions.
