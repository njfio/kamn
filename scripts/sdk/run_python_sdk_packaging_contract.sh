#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PACKAGING_PUBLISH_READINESS_REASON_TAXONOMY_VERSION="kamn.sdk.python-packaging-publish-readiness-reason-taxonomy.v1"
PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV="python_packaging_metadata_missing,python_packaging_metadata_invalid,python_packaging_import_probe_failed,python_packaging_unittest_contract_failed"

output_json=""
max_seconds=180

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be an integer" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"
pyproject_file="$ROOT_DIR/pyproject.toml"
if [ ! -f "$pyproject_file" ]; then
  echo "expected python sdk packaging metadata file: pyproject.toml" >&2
  exit 1
fi

python3 - "$pyproject_file" <<'PY'
import pathlib
import sys
import tomllib

pyproject_path = pathlib.Path(sys.argv[1])
payload = tomllib.loads(pyproject_path.read_text(encoding="utf-8"))

build_system = payload.get("build-system", {})
if build_system.get("build-backend") != "setuptools.build_meta":
    raise SystemExit("invalid python sdk build backend; expected setuptools.build_meta")

project = payload.get("project", {})
if project.get("name") != "kamn-sdk":
    raise SystemExit("invalid python sdk project.name; expected kamn-sdk")
if not project.get("version"):
    raise SystemExit("invalid python sdk project.version; expected non-empty string")
if project.get("requires-python") != ">=3.10":
    raise SystemExit("invalid python sdk requires-python; expected >=3.10")

tool = payload.get("tool", {})
setuptools_config = tool.get("setuptools", {})
modules = setuptools_config.get("py-modules", [])
if "kamn_sdk" not in modules:
    raise SystemExit("python sdk py-modules must include kamn_sdk")
PY

import_output="$(python3 - <<'PY'
from kamn_sdk import KAMNClient, LiveKAMNClient

client = KAMNClient()
did = client.register("autonomous", "claude-4", ["text"])
if not did.startswith("kamn:did:agent:"):
    raise SystemExit("unexpected DID contract from KAMNClient.register")

live = LiveKAMNClient("https://live.kamn.testnet/python-packaging-contract")
live_did = live.register("autonomous", "claude-4", ["text"])
if not live_did.startswith("kamn:did:agent:"):
    raise SystemExit("unexpected DID contract from LiveKAMNClient.register")

print("sdk_import_status=verified")
PY
)"
if ! printf '%s\n' "$import_output" | grep -q '^sdk_import_status=verified$'; then
  echo "expected python sdk import probe verification marker" >&2
  exit 1
fi

python_test_output="$({
  python3 -m unittest tests/python/test_sdk.py
} 2>&1)"
if ! printf '%s\n' "$python_test_output" | grep -q '^OK$'; then
  echo "expected python sdk unittest suite to pass" >&2
  printf '%s\n' "$python_test_output" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "python sdk packaging contract exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/python-sdk-packaging-contract-report.json"
cat >"$report_json" <<JSON
{
  "schema_version": "kamn.sdk.python-packaging-contract.v1",
  "status": "pass",
  "final_decision": "GO",
  "package_metadata_status": "verified",
  "sdk_import_status": "verified",
  "packaging_contract_status": "verified",
  "packaging_publish_readiness_reason_taxonomy_version": "${PACKAGING_PUBLISH_READINESS_REASON_TAXONOMY_VERSION}",
  "packaging_publish_readiness_reason_codes_csv": "${PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV}",
  "packaging_publish_readiness_reason_codes_value": "${PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV}",
  "packaging_publish_readiness_status": "verified",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "package_metadata_status=verified"
echo "sdk_import_status=verified"
echo "packaging_contract_status=verified"
echo "packaging_publish_readiness_reason_taxonomy_version=${PACKAGING_PUBLISH_READINESS_REASON_TAXONOMY_VERSION}"
echo "packaging_publish_readiness_reason_codes_csv=${PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV}"
echo "packaging_publish_readiness_status=verified"
