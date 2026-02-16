#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/sdk/validate_python_sdk_packaging_live.sh"
PYTHON_SDK_DOC="$ROOT_DIR/docs/sdk/python-sdk.md"
SDK_README_DOC="$ROOT_DIR/docs/sdk/README.md"
PACKAGING_PUBLISH_READINESS_REASON_TAXONOMY_VERSION="kamn.sdk.python-packaging-publish-readiness-reason-taxonomy.v1"
PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV="python_packaging_metadata_missing,python_packaging_metadata_invalid,python_packaging_import_probe_failed,python_packaging_unittest_contract_failed"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected python sdk packaging live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected python sdk packaging live pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected python sdk packaging live GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^packaging_contract_status=verified$'; then
  echo "expected python sdk packaging live packaging contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^evidence_bundle_status=verified$'; then
  echo "expected python sdk packaging live evidence bundle marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected python sdk packaging live fail-closed marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_reason_code=missing_pyproject$'; then
  echo "expected python sdk packaging live fail-closed reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^publish_readiness_taxonomy_status=verified$'; then
  echo "expected python sdk packaging live publish-readiness taxonomy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^packaging_publish_readiness_reason_taxonomy_version=${PACKAGING_PUBLISH_READINESS_REASON_TAXONOMY_VERSION}$"; then
  echo "expected python sdk packaging live publish-readiness taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^packaging_publish_readiness_reason_codes_csv=${PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV}$"; then
  echo "expected python sdk packaging live publish-readiness reason-codes marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.sdk.python-packaging-live-validation.v1":
    raise SystemExit("unexpected python sdk packaging live schema")
if payload.get("status") != "pass":
    raise SystemExit("expected live status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected live final_decision=GO")
if payload.get("packaging_contract_status") != "verified":
    raise SystemExit("expected packaging_contract_status=verified")
if payload.get("evidence_bundle_status") != "verified":
    raise SystemExit("expected evidence_bundle_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
if payload.get("fail_closed_reason_code") != "missing_pyproject":
    raise SystemExit("expected fail_closed_reason_code=missing_pyproject")
if payload.get("publish_readiness_taxonomy_status") != "verified":
    raise SystemExit("expected publish_readiness_taxonomy_status=verified")
if payload.get("packaging_publish_readiness_reason_taxonomy_version") != "kamn.sdk.python-packaging-publish-readiness-reason-taxonomy.v1":
    raise SystemExit("expected packaging_publish_readiness_reason_taxonomy_version marker")
if payload.get("packaging_publish_readiness_reason_codes_csv") != "python_packaging_metadata_missing,python_packaging_metadata_invalid,python_packaging_import_probe_failed,python_packaging_unittest_contract_failed":
    raise SystemExit("expected packaging_publish_readiness_reason_codes_csv marker")
PY

set +e
invalid_budget_output="$({ bash "$VALIDATION_SCRIPT" --max-seconds invalid; } 2>&1)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected python sdk packaging live validation script to reject invalid max-seconds" >&2
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
if [ ! -f "$SDK_README_DOC" ]; then
  echo "expected sdk README docs file for publish-readiness marker checks" >&2
  exit 1
fi
if ! grep -Fq "packaging_publish_readiness_reason_codes_csv=${PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV}" "$SDK_README_DOC"; then
  echo "expected sdk README to include publish-readiness reason-codes marker" >&2
  exit 1
fi

echo "python sdk packaging live validation tests passed."
