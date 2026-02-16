#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/sdk/run_python_sdk_packaging_contract.sh"
PYTHON_SDK_DOC="$ROOT_DIR/docs/sdk/python-sdk.md"
SDK_README_DOC="$ROOT_DIR/docs/sdk/README.md"
PACKAGING_PUBLISH_READINESS_REASON_TAXONOMY_VERSION="kamn.sdk.python-packaging-publish-readiness-reason-taxonomy.v1"
PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV="python_packaging_metadata_missing,python_packaging_metadata_invalid,python_packaging_import_probe_failed,python_packaging_unittest_contract_failed"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected python sdk packaging contract runner to be executable" >&2
  exit 1
fi

run_output="$(bash "$RUNNER" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$run_output" | grep -q '^status=pass$'; then
  echo "expected python sdk packaging contract pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^final_decision=GO$'; then
  echo "expected python sdk packaging contract GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^package_metadata_status=verified$'; then
  echo "expected python sdk packaging metadata marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^sdk_import_status=verified$'; then
  echo "expected python sdk import marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^packaging_contract_status=verified$'; then
  echo "expected python sdk packaging contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q "^packaging_publish_readiness_reason_taxonomy_version=${PACKAGING_PUBLISH_READINESS_REASON_TAXONOMY_VERSION}$"; then
  echo "expected python sdk packaging publish-readiness taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q "^packaging_publish_readiness_reason_codes_csv=${PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV}$"; then
  echo "expected python sdk packaging publish-readiness reason-codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^packaging_publish_readiness_status=verified$'; then
  echo "expected python sdk packaging publish-readiness status marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.sdk.python-packaging-contract.v1":
    raise SystemExit("unexpected python packaging schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("package_metadata_status") != "verified":
    raise SystemExit("expected package_metadata_status=verified")
if payload.get("sdk_import_status") != "verified":
    raise SystemExit("expected sdk_import_status=verified")
if payload.get("packaging_contract_status") != "verified":
    raise SystemExit("expected packaging_contract_status=verified")
if payload.get("packaging_publish_readiness_reason_taxonomy_version") != "kamn.sdk.python-packaging-publish-readiness-reason-taxonomy.v1":
    raise SystemExit("expected packaging_publish_readiness_reason_taxonomy_version marker")
if payload.get("packaging_publish_readiness_reason_codes_csv") != "python_packaging_metadata_missing,python_packaging_metadata_invalid,python_packaging_import_probe_failed,python_packaging_unittest_contract_failed":
    raise SystemExit("expected packaging_publish_readiness_reason_codes_csv marker")
if payload.get("packaging_publish_readiness_status") != "verified":
    raise SystemExit("expected packaging_publish_readiness_status=verified")
PY

set +e
invalid_budget_output="$({ bash "$RUNNER" --max-seconds invalid; } 2>&1)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected python sdk packaging contract runner to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be an integer'; then
  echo "expected deterministic invalid max-seconds marker" >&2
  exit 1
fi

if [ ! -f "$PYTHON_SDK_DOC" ]; then
  echo "expected python sdk docs file for publish-readiness marker checks" >&2
  exit 1
fi
if ! grep -Fq "packaging_publish_readiness_reason_taxonomy_version=${PACKAGING_PUBLISH_READINESS_REASON_TAXONOMY_VERSION}" "$PYTHON_SDK_DOC"; then
  echo "expected python sdk docs to include publish-readiness taxonomy version marker" >&2
  exit 1
fi
if ! grep -Fq "packaging_publish_readiness_reason_codes_csv=${PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV}" "$PYTHON_SDK_DOC"; then
  echo "expected python sdk docs to include publish-readiness reason-codes marker" >&2
  exit 1
fi
if [ ! -f "$SDK_README_DOC" ]; then
  echo "expected sdk README docs file for publish-readiness marker checks" >&2
  exit 1
fi
if ! grep -Fq "packaging_publish_readiness_reason_taxonomy_version=${PACKAGING_PUBLISH_READINESS_REASON_TAXONOMY_VERSION}" "$SDK_README_DOC"; then
  echo "expected sdk README to include publish-readiness taxonomy version marker" >&2
  exit 1
fi

echo "python sdk packaging contract runner tests passed."
