#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/sdk/run_python_sdk_packaging_contract.sh"
PACKAGING_PUBLISH_READINESS_REASON_TAXONOMY_VERSION="kamn.sdk.python-packaging-publish-readiness-reason-taxonomy.v1"
PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV="python_packaging_metadata_missing,python_packaging_metadata_invalid,python_packaging_import_probe_failed,python_packaging_unittest_contract_failed"

output_json=""
max_seconds=240

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
pyproject_file="$ROOT_DIR/pyproject.toml"
pyproject_backup="$TMP_DIR/pyproject.toml.backup"
pyproject_moved=0

cleanup() {
  if [ "$pyproject_moved" -eq 1 ] && [ -f "$pyproject_backup" ]; then
    mv "$pyproject_backup" "$pyproject_file"
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

start_epoch="$(date +%s)"
live_report="$TMP_DIR/python-sdk-packaging-live-report.json"
run_output="$({
  bash "$RUNNER" --max-seconds 180 --output-json "$live_report"
} 2>&1)"

if ! printf '%s\n' "$run_output" | grep -q '^status=pass$'; then
  echo "expected python sdk packaging contract pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^final_decision=GO$'; then
  echo "expected python sdk packaging contract GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^packaging_contract_status=verified$'; then
  echo "expected python sdk packaging contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q "^packaging_publish_readiness_reason_taxonomy_version=${PACKAGING_PUBLISH_READINESS_REASON_TAXONOMY_VERSION}$"; then
  echo "expected python sdk packaging contract publish-readiness taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q "^packaging_publish_readiness_reason_codes_csv=${PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV}$"; then
  echo "expected python sdk packaging contract publish-readiness reason-codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^packaging_publish_readiness_status=verified$'; then
  echo "expected python sdk packaging contract publish-readiness status marker" >&2
  exit 1
fi

if [ ! -f "$pyproject_file" ]; then
  echo "expected pyproject.toml for missing-file fail-closed drill" >&2
  exit 1
fi
mv "$pyproject_file" "$pyproject_backup"
pyproject_moved=1

set +e
fail_closed_output="$({
  bash "$RUNNER"
} 2>&1)"
fail_closed_code=$?
set -e

mv "$pyproject_backup" "$pyproject_file"
pyproject_moved=0

if [ "$fail_closed_code" -eq 0 ]; then
  echo "expected missing pyproject drill to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_closed_output" | grep -q 'expected python sdk packaging metadata file: pyproject.toml'; then
  echo "expected deterministic missing pyproject failure marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "python sdk packaging live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/python-sdk-packaging-live-validation-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.sdk.python-packaging-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "packaging_contract_status": "verified",
  "evidence_bundle_status": "verified",
  "publish_readiness_taxonomy_status": "verified",
  "packaging_publish_readiness_reason_taxonomy_version": "${PACKAGING_PUBLISH_READINESS_REASON_TAXONOMY_VERSION}",
  "packaging_publish_readiness_reason_codes_csv": "${PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV}",
  "packaging_publish_readiness_reason_codes_value": "${PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV}",
  "fail_closed_status": "verified",
  "fail_closed_reason_code": "missing_pyproject",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "packaging_contract_status=verified"
echo "evidence_bundle_status=verified"
echo "publish_readiness_taxonomy_status=verified"
echo "packaging_publish_readiness_reason_taxonomy_version=${PACKAGING_PUBLISH_READINESS_REASON_TAXONOMY_VERSION}"
echo "packaging_publish_readiness_reason_codes_csv=${PACKAGING_PUBLISH_READINESS_REASON_CODES_CSV}"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=missing_pyproject"
