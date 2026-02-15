#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_libp2p_three_node_discovery_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected libp2p three-node discovery policy checker script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/libp2p-three-node-discovery-summary.json"
cat >"$report_file" <<'JSON'
{
  "schema_version": "kamn.runtime.libp2p-three-node-discovery-live-report.v1",
  "status": "pass",
  "final_decision": "GO",
  "lane_mode": "dry-run",
  "node_count": 3,
  "three_node_discovery_status": "verified",
  "gossip_propagation_status": "verified",
  "lifecycle_transition_status": "verified",
  "runtime_transport_mode": "libp2p_discovery_gossip_three_node",
  "fail_closed_status": "verified",
  "ci_fast_gate_exclusion_status": "verified",
  "performance_budget_status": "verified",
  "execution_reason_code": "dry_run_no_commands_executed",
  "command_count": 0,
  "elapsed_seconds": 1
}
JSON

policy_report="$TMP_DIR/libp2p-three-node-discovery-policy.json"
policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected libp2p three-node discovery policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected libp2p three-node discovery policy checker GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^libp2p_three_node_discovery_policy_status=verified$'; then
  echo "expected libp2p three-node discovery policy checker status marker" >&2
  exit 1
fi

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.libp2p-three-node-discovery-live-policy-report.v1":
    raise SystemExit("unexpected libp2p three-node discovery policy report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("libp2p_three_node_discovery_policy_status") != "verified":
    raise SystemExit("expected libp2p_three_node_discovery_policy_status=verified")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected policy checker success reason code ['none']")
PY

tampered_report="$TMP_DIR/libp2p-three-node-discovery-summary.tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["three_node_discovery_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/libp2p-three-node-discovery-policy.tampered.json" 2>&1
)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered libp2p three-node discovery report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'libp2p_three_node_discovery_policy_marker_missing:three_node_discovery_status'; then
  echo "expected deterministic mismatch reason code for tampered libp2p policy validation" >&2
  exit 1
fi

echo "libp2p three-node discovery live policy checker tests passed."
