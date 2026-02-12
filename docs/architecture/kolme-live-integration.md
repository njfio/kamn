# Kolme Live Integration Architecture

This document captures the live-node integration contract surface between KAMN
runtime signing and `njfio/kolme_fork` compatibility expectations.

## Signature Parity Lane (Task #2299)

- Vector source policy:
  - `fixtures/kolme_commit/signature_parity_vectors.json`
  - schema: `kamn.kolme.signature-parity-vectors.v1`
  - source contract marker: `njfio/kolme_fork-secp256k1-v1`
- Parity matrix runner:
  - `python3 scripts/kolme/run_signature_parity_matrix.py --fixture fixtures/kolme_commit/signature_parity_vectors.json --output-json /tmp/kolme-signature-parity-matrix-report.json`
  - report schema: `kamn.kolme.signature-parity-matrix-report.v1`
  - runner executes deterministic adapter probe:
    - `cargo test -p kamn-node integration_kolme_live_signer_vector_probe_contract -- --ignored --nocapture`
- Policy checker:
  - `python3 scripts/kolme/check_signature_parity_policy.py --report-file /tmp/kolme-signature-parity-matrix-report.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-signature-parity-policy-report.json`
  - policy schema: `kamn.kolme.signature-parity-policy-report.v1`
- Contract lane wrapper:
  - `bash scripts/kolme/run_signature_parity_contract_lane.sh --output-json /tmp/kolme-signature-parity-matrix-report.json --policy-output-json /tmp/kolme-signature-parity-policy-report.json`

## Drift Handling

- Known-good vectors must produce `GO` parity outcomes.
- Known-bad vectors must produce `NO-GO` outcomes with deterministic reason codes
  (for example `parity_signature_mismatch`).
- Any probe failure without explicit mismatch reasons is classified as
  `parity_probe_failed` and treated as fail-closed.
