#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ASSET_CONTRACT_TEST="$ROOT_DIR/scripts/deploy/test_deployment_assets.sh"
ASSET_LIVE_VALIDATOR="$ROOT_DIR/scripts/deploy/validate_deployment_assets_live.sh"
CI_STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
DEPLOY_DOC="$ROOT_DIR/docs/ops/deployment.md"
DOCKER_DOC="$ROOT_DIR/docs/deployment/docker.md"
PACKAGING_REASON_TAXONOMY_VERSION="kamn.deploy.compose-packaging-reason-taxonomy.v1"
PACKAGING_REASON_CODES_CSV="compose_packaging_manifest_drift_detected,compose_packaging_config_drift_detected,compose_packaging_evidence_contract_drift_detected"
PACKAGING_CONTRACT_EVIDENCE_STATUS="verified"

output_json=""
max_seconds="${KAMN_COMPOSE_TOPOLOGY_CONTRACT_MAX_SECONDS:-240}"
ci_fast_gate="PASS"

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
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
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
if [ "$max_seconds" -le 0 ]; then
  echo "max-seconds must be greater than zero" >&2
  exit 1
fi
if [[ "$ci_fast_gate" != "PASS" && "$ci_fast_gate" != "FAIL" ]]; then
  echo "ci-fast-gate must be PASS or FAIL" >&2
  exit 1
fi

for required_exec in "$ASSET_CONTRACT_TEST" "$ASSET_LIVE_VALIDATOR"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected required executable script '$required_exec'" >&2
    exit 1
  fi
done
for required_doc in "$CI_STRATEGY_DOC" "$DEPLOY_DOC" "$DOCKER_DOC"; do
  if [ ! -f "$required_doc" ]; then
    echo "expected required documentation file '$required_doc'" >&2
    exit 1
  fi
done

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

bash "$ASSET_CONTRACT_TEST"

live_report="$TMP_DIR/deployment-assets-live-validation-report.json"
live_output="$(
  bash "$ASSET_LIVE_VALIDATOR" \
    --output-json "$live_report" \
    --max-seconds "$max_seconds"
)"
if ! printf '%s\n' "$live_output" | grep -q '^status=pass$'; then
  echo "expected deployment assets live validation status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$live_output" | grep -q '^final_decision=GO$'; then
  echo "expected deployment assets live validation final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$live_output" | grep -q '^asset_contract_status=verified$'; then
  echo "expected deployment assets live validation asset contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$live_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected deployment assets live validation fail-closed marker" >&2
  exit 1
fi
if ! printf '%s\n' "$live_output" | grep -q '^compose_manifest_contract_status=verified$'; then
  echo "expected deployment assets live validation compose-manifest marker" >&2
  exit 1
fi
if ! printf '%s\n' "$live_output" | grep -q '^compose_config_contract_status=verified$'; then
  echo "expected deployment assets live validation compose-config marker" >&2
  exit 1
fi
if ! printf '%s\n' "$live_output" | grep -q '^k8s_manifest_contract_status=verified$'; then
  echo "expected deployment assets live validation k8s-manifest marker" >&2
  exit 1
fi

for required_ref in \
  "validate_compose_topology_contract_lane.sh" \
  "check_compose_topology_contract_policy.sh" \
  "test_validate_compose_topology_contract_lane.sh" \
  "test_check_compose_topology_contract_policy.sh"; do
  if ! grep -q "$required_ref" "$CI_STRATEGY_DOC"; then
    echo "expected CI strategy docs to reference $required_ref" >&2
    exit 1
  fi
done
if ! grep -Fq "packaging_reason_taxonomy_version=${PACKAGING_REASON_TAXONOMY_VERSION}" "$CI_STRATEGY_DOC"; then
  echo "expected CI strategy docs to include compose topology packaging reason taxonomy marker" >&2
  exit 1
fi
if ! grep -Fq "packaging_reason_codes_csv=${PACKAGING_REASON_CODES_CSV}" "$CI_STRATEGY_DOC"; then
  echo "expected CI strategy docs to include compose topology packaging reason codes marker" >&2
  exit 1
fi
if ! grep -Fq "packaging_contract_evidence_status=verified" "$DEPLOY_DOC"; then
  echo "expected deployment docs to include packaging contract evidence marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "compose topology contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

summary_report="$TMP_DIR/compose-topology-contract-lane-summary.json"
python3 - "$summary_report" "$elapsed_seconds" "$max_seconds" "$ci_fast_gate" "$PACKAGING_REASON_TAXONOMY_VERSION" "$PACKAGING_REASON_CODES_CSV" "$PACKAGING_CONTRACT_EVIDENCE_STATUS" <<'PY'
import json
import pathlib
import sys

summary_report_file = pathlib.Path(sys.argv[1])
elapsed_seconds = int(sys.argv[2])
max_seconds = int(sys.argv[3])
ci_fast_gate = sys.argv[4]
packaging_reason_taxonomy_version = sys.argv[5]
packaging_reason_codes_csv = sys.argv[6]
packaging_contract_evidence_status = sys.argv[7]

payload = {
    "schema_version": "kamn.deploy.compose-topology-contract-lane-summary.v1",
    "status": "pass",
    "final_decision": "GO",
    "ci_fast_gate": ci_fast_gate,
    "compose_runtime_mode_full_status": "verified",
    "compose_api_port_status": "verified",
    "compose_volume_network_status": "verified",
    "compose_docs_parity_status": "verified",
    "compose_manifest_contract_status": "verified",
    "compose_config_contract_status": "verified",
    "k8s_manifest_contract_status": "verified",
    "packaging_reason_taxonomy_version": packaging_reason_taxonomy_version,
    "packaging_reason_codes_csv": packaging_reason_codes_csv,
    "packaging_reason_codes_value": packaging_reason_codes_csv,
    "packaging_contract_evidence_status": packaging_contract_evidence_status,
    "fail_closed_status": "verified",
    "reason_code": "compose_topology_contract_verified",
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
}
summary_report_file.write_text(
    json.dumps(payload, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

if [[ -n "$output_json" ]]; then
  cp "$summary_report" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "ci_fast_gate=${ci_fast_gate}"
echo "compose_runtime_mode_full_status=verified"
echo "compose_api_port_status=verified"
echo "compose_volume_network_status=verified"
echo "compose_docs_parity_status=verified"
echo "compose_manifest_contract_status=verified"
echo "compose_config_contract_status=verified"
echo "k8s_manifest_contract_status=verified"
echo "packaging_reason_taxonomy_version=${PACKAGING_REASON_TAXONOMY_VERSION}"
echo "packaging_reason_codes_csv=${PACKAGING_REASON_CODES_CSV}"
echo "packaging_contract_evidence_status=${PACKAGING_CONTRACT_EVIDENCE_STATUS}"
echo "fail_closed_status=verified"
echo "reason_code=compose_topology_contract_verified"
