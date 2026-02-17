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

bad_compose="$TMP_DIR/docker-compose.healthcheck.bad.yml"
sed 's|19081/healthz|19081/healthz-drift|g' "$ROOT_DIR/deploy/docker-compose.yml" >"$bad_compose"

set +e
healthcheck_negative_output="$(
  COMPOSE_FILE_PATH="$bad_compose" \
  bash "$ROOT_DIR/scripts/deploy/test_deployment_assets.sh" 2>&1
)"
healthcheck_negative_code=$?
set -e
if [ "$healthcheck_negative_code" -eq 0 ]; then
  echo "expected deployment assets checker to fail closed for invalid compose healthcheck marker" >&2
  exit 1
fi
if ! printf '%s\n' "$healthcheck_negative_output" | grep -q 'expected docker-compose processor healthcheck probe marker'; then
  printf '%s\n' "$healthcheck_negative_output" >&2
  echo "expected deterministic fail-closed reason marker for invalid compose healthcheck marker" >&2
  exit 1
fi

bad_compose_runtime_mode="$TMP_DIR/docker-compose.runtime-mode.bad.yml"
sed 's/--runtime-mode/--runtime_mode_drift/g' "$ROOT_DIR/deploy/docker-compose.yml" >"$bad_compose_runtime_mode"

set +e
runtime_mode_negative_output="$(
  COMPOSE_FILE_PATH="$bad_compose_runtime_mode" \
  bash "$ROOT_DIR/scripts/deploy/test_deployment_assets.sh" 2>&1
)"
runtime_mode_negative_code=$?
set -e
if [ "$runtime_mode_negative_code" -eq 0 ]; then
  echo "expected deployment assets checker to fail closed for invalid compose runtime-mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$runtime_mode_negative_output" | grep -q 'expected docker-compose runtime mode command marker'; then
  printf '%s\n' "$runtime_mode_negative_output" >&2
  echo "expected deterministic fail-closed reason marker for invalid compose runtime-mode marker" >&2
  exit 1
fi

bad_manifest="$TMP_DIR/kamn-node.manifest.bad.yaml"
grep -v 'KAMN_NODE_DAEMON_MAX_TICKS' "$ROOT_DIR/deploy/k8s/kamn-node.yaml" >"$bad_manifest"

set +e
manifest_negative_output="$(
  K8S_MANIFEST_PATH="$bad_manifest" \
  bash "$ROOT_DIR/scripts/deploy/test_deployment_assets.sh" 2>&1
)"
manifest_negative_code=$?
set -e
if [ "$manifest_negative_code" -eq 0 ]; then
  echo "expected deployment assets checker to fail closed for invalid k8s manifest marker" >&2
  exit 1
fi
if ! printf '%s\n' "$manifest_negative_output" | grep -q 'expected kubernetes manifest daemon max-ticks env marker'; then
  printf '%s\n' "$manifest_negative_output" >&2
  echo "expected deterministic fail-closed reason marker for invalid k8s manifest marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "deployment assets live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/deployment-assets-live-validation-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.deploy.assets.live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "asset_contract_status": "verified",
  "fail_closed_status": "verified",
  "compose_manifest_contract_status": "verified",
  "compose_config_contract_status": "verified",
  "k8s_manifest_contract_status": "verified",
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
echo "compose_manifest_contract_status=verified"
echo "compose_config_contract_status=verified"
echo "k8s_manifest_contract_status=verified"
