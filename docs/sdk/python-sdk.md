# Python SDK Packaging Contract

This document defines the production packaging contract for the KAMN Python SDK.

## Scope

The Python SDK surface is provided by module `kamn_sdk.py` and includes:

- in-memory client (`KAMNClient`)
- live transport client (`LiveKAMNClient`)
- transport error taxonomy and parity contracts

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
