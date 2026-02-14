#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

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

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

pushd "$ROOT_DIR" >/dev/null
bash scripts/deploy/test_deployment_assets.sh
popd >/dev/null

bad_dockerfile="$TMP_DIR/Dockerfile.bad"
grep -v '^FROM rust:' "$ROOT_DIR/Dockerfile" >"$bad_dockerfile"

set +e
negative_output="$(DOCKERFILE_PATH="$bad_dockerfile" bash "$ROOT_DIR/scripts/deploy/test_deployment_assets.sh" 2>&1)"
negative_code=$?
set -e
if [ "$negative_code" -eq 0 ]; then
  echo "expected deployment assets checker to fail closed for invalid Dockerfile" >&2
  exit 1
fi
if ! printf '%s\n' "$negative_output" | grep -q 'expected Dockerfile multi-stage builder image marker'; then
  printf '%s\n' "$negative_output" >&2
  echo "expected deterministic fail-closed reason marker for invalid Dockerfile" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "deployment assets live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/deployment-assets-live-validation-report.json"
cat >"$report_json" <<JSON
{
  "schema_version": "kamn.deploy.assets.live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "asset_contract_status": "verified",
  "fail_closed_status": "verified",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "asset_contract_status=verified"
echo "fail_closed_status=verified"
