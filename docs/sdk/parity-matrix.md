# SDK Parity Matrix

This document defines the cross-language SDK parity matrix contract for Rust, Python, and TypeScript surfaces.

## Scope

The matrix validates two parity layers:

- Register payload validation parity (`scripts/sdk/run_sdk_parity_matrix.sh`)
- Live transport contract parity (`scripts/sdk/run_live_transport_parity_contract_lane.sh`)

Unified runner:

- `scripts/sdk/run_cross_language_sdk_parity_matrix.sh`

## Runner Contract

```bash
bash scripts/sdk/run_cross_language_sdk_parity_matrix.sh \
  --mode contract \
  --languages all \
  --fixture fixtures/sdk_parity/register_validation_cases.json \
  --output-json /tmp/cross-language-sdk-parity-report.json
```

Arguments:

- `--mode`: `contract` (default) or `deep`
- `--languages`: language selector for contract mode (`all`, or comma-separated subset)
- `--fixture`: register parity fixture JSON path
- `--output-json`: optional report output path
- `--max-seconds`: runtime budget guard (default `180`)

Deterministic success markers:

- `status=pass`
- `final_decision=GO`
- `mode=<contract|deep>`
- `register_parity_status=verified`
- `live_transport_parity_status=verified`

JSON schema marker:

- `schema_version=kamn.sdk.cross-language-parity.v1`

## Fail-Closed Guardrails

The runner fails closed for:

- invalid mode (`mode must be one of: contract,deep`)
- invalid runtime budget (`max-seconds must be an integer`)
- unsupported deep-mode language override (`deep mode only supports languages=all`)
- upstream parity runner failures

## Validation Commands

Core harness:

- `bash scripts/sdk/test_run_cross_language_sdk_parity_matrix.sh`

Supporting parity harnesses:

- `bash scripts/sdk/test_run_sdk_parity_matrix.sh`
- `bash scripts/sdk/test_run_live_transport_parity_contract_lane.sh`
