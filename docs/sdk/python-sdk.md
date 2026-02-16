# Python SDK Packaging Contract

This document defines the production packaging contract for the KAMN Python SDK.

## Scope

The Python SDK surface is provided by module `kamn_sdk.py` and includes:

- in-memory client (`KAMNClient`)
- live transport client (`LiveKAMNClient`)
- transport error taxonomy and parity contracts

Live transport backend adapter errors use a normalized envelope contract:

- preferred adapter error shape: `{ "status": "error", "reason_code": "...", "message": "..." }`
- legacy adapter error shape `{ "status": "error", "reason": "..." }` remains supported
- raised exception: `LiveTransportBackendAdapterError` exposes:
  - `operation`
  - `reason_code`
  - `message`
  - backward-compatible alias `reason` (mapped to `reason_code`)

Packaging metadata is published through repository-root `pyproject.toml`.

## Packaging Metadata

Required project contract:

- `project.name = "kamn-sdk"`
- `project.version` is non-empty
- `project.requires-python = ">=3.10"`
- build backend is `setuptools.build_meta`
- `tool.setuptools.py-modules` includes `kamn_sdk`

## Packaging Contract Runner

```bash
bash scripts/sdk/run_python_sdk_packaging_contract.sh --output-json /tmp/python-sdk-packaging-report.json
```

Deterministic success markers:

- `status=pass`
- `final_decision=GO`
- `package_metadata_status=verified`
- `sdk_import_status=verified`
- `packaging_contract_status=verified`
- `packaging_publish_readiness_reason_taxonomy_version=kamn.sdk.python-packaging-publish-readiness-reason-taxonomy.v1`
- `packaging_publish_readiness_reason_codes_csv=python_packaging_metadata_missing,python_packaging_metadata_invalid,python_packaging_import_probe_failed,python_packaging_unittest_contract_failed`
- `packaging_publish_readiness_status=verified`

JSON schema marker:

- `schema_version=kamn.sdk.python-packaging-contract.v1`

## Validation Harness

```bash
bash scripts/sdk/test_run_python_sdk_packaging_contract.sh
```

Fail-closed guardrails:

- invalid runtime budget: `max-seconds must be an integer`
- missing/invalid packaging metadata in `pyproject.toml`
- SDK import/contract probe drift
- Python SDK unit suite drift (`python3 -m unittest tests/python/test_sdk.py`)

## Live Validation

Live validation lane:

- `bash scripts/sdk/test_validate_python_sdk_packaging_live.sh`
- `bash scripts/sdk/validate_python_sdk_packaging_live.sh --output-json /tmp/python-sdk-packaging-live-report.json`

Deterministic success markers:

- `status=pass`
- `final_decision=GO`
- `packaging_contract_status=verified`
- `evidence_bundle_status=verified`
- `publish_readiness_taxonomy_status=verified`
- `packaging_publish_readiness_reason_taxonomy_version=kamn.sdk.python-packaging-publish-readiness-reason-taxonomy.v1`
- `packaging_publish_readiness_reason_codes_csv=python_packaging_metadata_missing,python_packaging_metadata_invalid,python_packaging_import_probe_failed,python_packaging_unittest_contract_failed`
- `fail_closed_status=verified`
- `fail_closed_reason_code=missing_pyproject`

Deterministic fail-closed drill:

- injected fault: temporarily remove `pyproject.toml` during runner invocation
- expected failure marker: `expected python sdk packaging metadata file: pyproject.toml`
