#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/deploy/validate_local_compose_multinode_live_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected local compose multinode contract lane script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/local-compose-multinode-contract-lane-report.json"
policy_report="$TMP_DIR/local-compose-multinode-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --ci-fast-gate PASS \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected local compose multinode contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected local compose multinode contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected local compose multinode contract lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^local_compose_multinode_contract_status=verified$'; then
  echo "expected local compose multinode contract lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^local_compose_multinode_policy_status=verified$'; then
  echo "expected local compose multinode contract lane policy status marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.deploy.local-compose-multinode-live-contract-lane-report.v1":
    raise SystemExit("unexpected local compose multinode contract lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("local_compose_multinode_contract_status") != "verified":
    raise SystemExit("expected local_compose_multinode_contract_status=verified")
if lane_payload.get("local_compose_multinode_policy_status") != "verified":
    raise SystemExit("expected local_compose_multinode_policy_status=verified")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.deploy.local-compose-multinode-live-policy-report.v1":
    raise SystemExit("unexpected local compose multinode policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if policy_payload.get("local_compose_multinode_policy_status") != "verified":
    raise SystemExit("expected local_compose_multinode_policy_status=verified in policy report")
PY

echo "local compose multinode contract lane tests passed."
