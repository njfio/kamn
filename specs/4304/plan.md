# Plan — #4304

Status: Reviewed

## Approach

- Extend `test_generate_gonogo_evidence_bundle.sh` with failing assertions for missing/stale/malformed TLS evidence.
- Reuse temporary fixture generation to keep tests deterministic and fast.
- Assert exact reason marker values to enforce deterministic normalization.

## Affected Areas

- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`

## Risks and Mitigations

- Risk: shell test fixture timestamps are time-sensitive.
  - Mitigation: use explicit fixed stale timestamps and bounded max-age values.
- Risk: too-broad assertions create brittle tests.
  - Mitigation: assert stable marker names + reason codes only.

## Interfaces and Contracts

- TLS evidence reason markers in generated bundle output.
