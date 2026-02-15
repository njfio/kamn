#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_libp2p_process_isolated_harness.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_libp2p_process_isolated_harness_policy.sh"

output_json=""
policy_output_json=""
max_seconds="${KAMN_LIBP2P_PROCESS_ISOLATED_HARNESS_CONTRACT_MAX_SECONDS:-240}"
mode="dry-run"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --policy-output-json)
      policy_output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    --mode)
      mode="${2:-}"
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
if [[ "$mode" != "dry-run" && "$mode" != "run" ]]; then
  echo "mode must be dry-run or run" >&2
  exit 1
fi

for required_exec in "$VALIDATION_SCRIPT" "$POLICY_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected required executable script '$required_exec'" >&2
    exit 1
  fi
done

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

summary_report="$TMP_DIR/libp2p-process-isolated-harness-summary.json"
policy_report="$TMP_DIR/libp2p-process-isolated-harness-policy.json"
tampered_report="$TMP_DIR/libp2p-process-isolated-harness-summary.tampered.json"

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode "$mode" \
    --max-seconds "$max_seconds" \
    --output-json "$summary_report"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected process-isolated harness validation status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected process-isolated harness validation final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^lane_mode=$mode$"; then
  echo "expected process-isolated harness validation lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^two_node_startup_status=verified$'; then
  echo "expected process-isolated harness two-node startup marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^three_node_startup_status=verified$'; then
  echo "expected process-isolated harness three-node startup marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^partition_rejoin_status=verified$'; then
  echo "expected process-isolated harness partition/rejoin marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^publish_drop_recovery_status=verified$'; then
  echo "expected process-isolated harness publish-drop marker" >&2
  exit 1
fi

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected process-isolated harness policy status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected process-isolated harness policy final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^libp2p_process_isolated_harness_policy_status=verified$'; then
  echo "expected process-isolated harness policy status marker" >&2
  exit 1
fi

cp "$summary_report" "$tampered_report"
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
tampered_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --output-json "$TMP_DIR/libp2p-process-isolated-harness-policy.tampered.json" 2>&1
)"
tampered_policy_code=$?
set -e
if [ "$tampered_policy_code" -eq 0 ]; then
  echo "expected tampered process-isolated harness report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_policy_output" | grep -q 'libp2p_process_isolated_harness_policy_marker_missing:partition_rejoin_status'; then
  echo "expected deterministic fail-closed reason for tampered process-isolated harness report" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "process-isolated harness contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

lane_report="$TMP_DIR/libp2p-process-isolated-harness-contract-lane-report.json"
python3 - "$summary_report" "$policy_report" "$lane_report" "$elapsed_seconds" "$max_seconds" "$mode" <<'PY'
import json
import pathlib
import sys

summary_report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy_report = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
lane_report_file = pathlib.Path(sys.argv[3])
elapsed_seconds = int(sys.argv[4])
max_seconds = int(sys.argv[5])
mode = sys.argv[6]

if summary_report.get("schema_version") != "kamn.runtime.libp2p-process-isolated-harness-report.v1":
    raise SystemExit("unexpected process-isolated harness summary schema")
if policy_report.get("schema_version") != "kamn.runtime.libp2p-process-isolated-harness-policy-report.v1":
    raise SystemExit("unexpected process-isolated harness policy schema")
if summary_report.get("final_decision") != "GO":
    raise SystemExit("expected process-isolated harness summary final_decision=GO")
if policy_report.get("final_decision") != "GO":
    raise SystemExit("expected process-isolated harness policy final_decision=GO")

lane_report = {
    "schema_version": "kamn.runtime.libp2p-process-isolated-harness-contract-lane-report.v1",
    "status": "pass",
    "final_decision": "GO",
    "lane_mode": mode,
    "libp2p_process_isolated_harness_contract_status": "verified",
    "libp2p_process_isolated_harness_policy_status": policy_report.get(
        "libp2p_process_isolated_harness_policy_status"
    ),
    "fail_closed_status": "verified",
    "fail_closed_reason_code": "libp2p_process_isolated_harness_policy_marker_missing:partition_rejoin_status",
    "performance_budget_status": "verified",
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
}
lane_report_file.write_text(json.dumps(lane_report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

if [[ -n "$output_json" ]]; then
  cp "$lane_report" "$output_json"
fi
if [[ -n "$policy_output_json" ]]; then
  cp "$policy_report" "$policy_output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "lane_mode=$mode"
echo "libp2p_process_isolated_harness_contract_status=verified"
echo "libp2p_process_isolated_harness_policy_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=libp2p_process_isolated_harness_policy_marker_missing:partition_rejoin_status"
echo "performance_budget_status=verified"
