#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_libp2p_process_isolated_harness_policy.sh"

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected process-isolated harness policy checker script to be executable" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
evidence_file="$TMP_DIR/process-harness-evidence.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$evidence_file" <<'JSON'
{
  "schema_version": "kamn.runtime.process-harness-evidence.v1",
  "status": "pass",
  "final_decision": "GO",
  "reason_code": "libp2p_process_isolated_harness_verified",
  "ports": {},
  "processes": [
    {"name": "two_node_handshake_discovery_gossip", "status": "skipped_dry_run"},
    {"name": "three_node_partition_rejoin_publish_drop", "status": "skipped_dry_run"}
  ],
  "artifacts": {}
}
JSON

report_file="$TMP_DIR/libp2p-process-isolated-harness-summary.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_file" <<JSON
{
  "schema_version": "kamn.runtime.libp2p-process-isolated-harness-report.v1",
  "status": "pass",
  "final_decision": "GO",
  "lane_mode": "dry-run",
  "runtime_transport_mode": "libp2p_process_isolated_convergence",
  "two_node_startup_status": "verified",
  "three_node_startup_status": "verified",
  "partition_rejoin_status": "verified",
  "publish_drop_recovery_status": "verified",
  "convergence_reason_codes": ["fork_choice_stale_block_height"],
  "topology_node_counts": [2, 3],
  "process_harness_evidence_file": "$evidence_file",
  "execution_reason_code": "dry_run_no_commands_executed",
  "command_count": 0,
  "elapsed_seconds": 0
}
JSON

policy_report="$TMP_DIR/libp2p-process-isolated-harness-policy.json"
policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected process-isolated harness policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected process-isolated harness policy checker GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^libp2p_process_isolated_harness_policy_status=verified$'; then
  echo "expected process-isolated harness policy checker status marker" >&2
  exit 1
fi

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.libp2p-process-isolated-harness-policy-report.v1":
    raise SystemExit("unexpected process-isolated harness policy report schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if payload.get("libp2p_process_isolated_harness_policy_status") != "verified":
    raise SystemExit("expected libp2p_process_isolated_harness_policy_status=verified")
PY

tampered_report="$TMP_DIR/libp2p-process-isolated-harness-summary.tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["partition_rejoin_status"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --output-json "$TMP_DIR/libp2p-process-isolated-harness-policy.tampered.json" 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered process-isolated harness report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'libp2p_process_isolated_harness_policy_marker_missing:partition_rejoin_status'; then
  echo "expected deterministic mismatch reason code for tampered process-isolated harness policy validation" >&2
  exit 1
fi

echo "process-isolated libp2p harness policy checker tests passed."
