#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_libp2p_three_node_discovery_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_libp2p_three_node_discovery_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_libp2p_three_node_discovery_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected libp2p three-node discovery contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected libp2p three-node discovery validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected libp2p three-node discovery policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/libp2p-three-node-discovery-contract-lane-report.json"
policy_report="$TMP_DIR/libp2p-three-node-discovery-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected libp2p three-node discovery contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected libp2p three-node discovery contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected libp2p three-node discovery contract lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^libp2p_three_node_discovery_contract_status=verified$'; then
  echo "expected libp2p three-node discovery contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^libp2p_three_node_discovery_policy_status=verified$'; then
  echo "expected libp2p three-node discovery policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=libp2p_three_node_discovery_policy_marker_missing:three_node_discovery_status$'; then
  echo "expected deterministic fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.libp2p-three-node-discovery-live-contract-lane-report.v1":
    raise SystemExit("unexpected libp2p three-node discovery contract-lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("libp2p_three_node_discovery_contract_status") != "verified":
    raise SystemExit("expected libp2p_three_node_discovery_contract_status=verified")
if lane_payload.get("libp2p_three_node_discovery_policy_status") != "verified":
    raise SystemExit("expected libp2p_three_node_discovery_policy_status=verified")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if lane_payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.libp2p-three-node-discovery-live-policy-report.v1":
    raise SystemExit("unexpected libp2p three-node discovery policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if policy_payload.get("libp2p_three_node_discovery_policy_status") != "verified":
    raise SystemExit("expected libp2p_three_node_discovery_policy_status=verified in policy report")
PY

set +e
invalid_ci_fast_gate_output="$(
  bash "$CONTRACT_LANE" \
    --ci-fast-gate MAYBE 2>&1
)"
invalid_ci_fast_gate_code=$?
set -e
if [ "$invalid_ci_fast_gate_code" -eq 0 ]; then
  echo "expected libp2p three-node discovery contract lane to reject invalid ci-fast-gate value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_ci_fast_gate_output" | grep -q 'ci-fast-gate must be PASS or FAIL'; then
  echo "expected deterministic invalid ci-fast-gate marker" >&2
  exit 1
fi

set +e
blocked_fast_gate_output="$(
  bash "$CONTRACT_LANE" \
    --ci-fast-gate FAIL 2>&1
)"
blocked_fast_gate_code=$?
set -e
if [ "$blocked_fast_gate_code" -eq 0 ]; then
  echo "expected libp2p three-node discovery contract lane to fail closed when ci-fast-gate=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$blocked_fast_gate_output" | grep -q 'ci_fast_gate_failed'; then
  echo "expected deterministic ci_fast_gate_failed marker" >&2
  exit 1
fi

echo "libp2p three-node discovery contract lane tests passed."
