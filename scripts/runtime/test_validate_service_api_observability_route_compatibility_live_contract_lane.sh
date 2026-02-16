#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_service_api_observability_route_compatibility_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_observability_route_compatibility_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_observability_route_compatibility_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected service api observability route compatibility contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected service api observability route compatibility validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected service api observability route compatibility policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/service-api-observability-route-compatibility-contract-lane-report.json"
policy_report="$TMP_DIR/service-api-observability-route-compatibility-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --max-seconds 180 \
    --command-max-seconds 60 \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
for marker in \
  '^status=pass$' \
  '^final_decision=GO$' \
  '^lane_mode=dry-run$' \
  '^service_api_observability_route_compatibility_contract_status=verified$' \
  '^service_api_observability_route_compatibility_policy_status=verified$' \
  '^route_parity_checkpoint_status=verified$' \
  '^fail_closed_checkpoint_status=verified$' \
  '^fail_closed_tamper_status=verified$' \
  '^docs_contract_status=verified$' \
  '^performance_budget_status=verified$' \
  '^fail_closed_reason_code=service_api_observability_route_compatibility_policy_matrix_row_status_mismatch:api_healthz_get$'; do
  if ! printf '%s\n' "$lane_output" | grep -q "$marker"; then
    echo "expected service api observability route compatibility contract lane marker: $marker" >&2
    exit 1
  fi
done

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.service-api-observability-route-compatibility-live-contract-lane-report.v1":
    raise SystemExit("unexpected contract lane schema for service api observability route compatibility")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("service_api_observability_route_compatibility_contract_status") != "verified":
    raise SystemExit("expected compatibility contract status=verified")
if lane_payload.get("service_api_observability_route_compatibility_policy_status") != "verified":
    raise SystemExit("expected compatibility policy status=verified")
if lane_payload.get("route_parity_checkpoint_status") != "verified":
    raise SystemExit("expected route_parity_checkpoint_status=verified")
if lane_payload.get("fail_closed_checkpoint_status") != "verified":
    raise SystemExit("expected fail_closed_checkpoint_status=verified")
if lane_payload.get("fail_closed_tamper_status") != "verified":
    raise SystemExit("expected fail_closed_tamper_status=verified")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.service-api-observability-route-compatibility-live-policy-report.v1":
    raise SystemExit("unexpected policy report schema in contract lane")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract-lane policy final_decision=GO")
if policy_payload.get("service_api_observability_route_compatibility_policy_status") != "verified":
    raise SystemExit("expected policy status marker in contract-lane policy report")
PY

set +e
blocked_fast_gate_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --ci-fast-gate FAIL 2>&1
)"
blocked_fast_gate_code=$?
set -e
if [ "$blocked_fast_gate_code" -eq 0 ]; then
  echo "expected service api observability route compatibility contract lane to fail closed when ci-fast-gate=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$blocked_fast_gate_output" | grep -q 'ci_fast_gate_failed'; then
  echo "expected deterministic ci_fast_gate_failed marker for service api observability route compatibility contract lane" >&2
  exit 1
fi

echo "service api observability route compatibility contract lane tests passed."
