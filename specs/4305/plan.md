# Plan — #4305

Status: Reviewed

## Approach

- Refactor TLS gate output normalization in `gonogo_evidence_contract.py` to produce deterministic reason strings.
- Wire normalized outputs through bundle generation and lane contract scripts.
- Keep changes additive and backward-compatible for existing marker consumers.

## Affected Areas

- `scripts/deploy/gonogo_evidence_contract.py`
- `scripts/deploy/gonogo_evidence_contract_lane_contract.sh`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`

## Risks and Mitigations

- Risk: marker format drift breaks existing lane tests.
  - Mitigation: preserve keys and validate using existing + new assertions.
- Risk: duplicate/unsorted reason codes degrade determinism.
  - Mitigation: canonicalize reason list before serialization.

## Interfaces and Contracts

- Deterministic TLS marker projection: `tls_evidence_reason_codes_csv` and `tls_evidence_reason_codes_value`.
