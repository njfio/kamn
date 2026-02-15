#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_full_runtime_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_local_full_runtime_live_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"

output_json=""
policy_output_json=""
max_seconds="${KAMN_LOCAL_FULL_RUNTIME_LIVE_CONTRACT_MAX_SECONDS:-240}"
ci_fast_gate="PASS"
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
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
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
if [[ "$ci_fast_gate" != "PASS" && "$ci_fast_gate" != "FAIL" ]]; then
  echo "ci-fast-gate must be PASS or FAIL" >&2
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
if [ ! -f "$STRATEGY_DOC" ]; then
  echo "expected required documentation file '$STRATEGY_DOC'" >&2
  exit 1
fi

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

summary_report="$TMP_DIR/local-full-runtime-live-summary.json"
policy_report="$TMP_DIR/local-full-runtime-live-policy.json"
tampered_report="$TMP_DIR/local-full-runtime-live-summary.tampered.json"

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode "$mode" \
    --max-seconds "$max_seconds" \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$summary_report"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected local full-runtime live validation status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected local full-runtime live validation final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fast_gate_exclusion_status=verified$'; then
  echo "expected local full-runtime live validation fast-gate exclusion marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^full_runtime_bootstrap_status=verified$'; then
  echo "expected local full-runtime live validation bootstrap marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^full_runtime_shutdown_status=verified$'; then
  echo "expected local full-runtime live validation shutdown marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^three_node_role_set_status=verified$'; then
  echo "expected local full-runtime live validation three-node role-set marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^transport_propagation_status=verified$'; then
  echo "expected local full-runtime live validation transport propagation marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^canonical_convergence_status=verified$'; then
  echo "expected local full-runtime live validation canonical convergence marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_transport_mode=libp2p_transport_fed$'; then
  echo "expected local full-runtime live validation runtime transport mode marker" >&2
  exit 1
fi

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected local full-runtime live policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected local full-runtime live policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^local_full_runtime_policy_status=verified$'; then
  echo "expected local full-runtime live policy checker status marker" >&2
  exit 1
fi

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["fast_gate_exclusion_status"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$TMP_DIR/local-full-runtime-live-policy.tampered.json" 2>&1
)"
tampered_policy_code=$?
set -e
if [ "$tampered_policy_code" -eq 0 ]; then
  echo "expected tampered local full-runtime live report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_policy_output" | grep -q 'local_full_runtime_policy_fast_gate_exclusion_mismatch'; then
  echo "expected deterministic fail-closed reason for tampered local full-runtime live report" >&2
  exit 1
fi

for required_ref in \
  "validate_local_full_runtime_live.sh" \
  "check_local_full_runtime_live_policy.sh" \
  "validate_local_full_runtime_live_contract_lane.sh" \
  "test_validate_local_full_runtime_live.sh" \
  "test_check_local_full_runtime_live_policy.sh" \
  "test_validate_local_full_runtime_live_contract_lane.sh"; do
  if ! grep -q "$required_ref" "$STRATEGY_DOC"; then
    echo "expected CI strategy docs to reference $required_ref" >&2
    exit 1
  fi
done
if ! grep -q "local full-runtime run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode." "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include local full-runtime run-mode exclusion marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "local full-runtime contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

lane_report="$TMP_DIR/local-full-runtime-live-contract-lane-report.json"
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

if summary_report.get("schema_version") != "kamn.runtime.local-full-runtime-live-report.v1":
    raise SystemExit("unexpected local full-runtime live summary schema")
if policy_report.get("schema_version") != "kamn.runtime.local-full-runtime-live-policy-report.v1":
    raise SystemExit("unexpected local full-runtime live policy schema")
if summary_report.get("final_decision") != "GO":
    raise SystemExit("expected local full-runtime live summary final_decision=GO")
if policy_report.get("final_decision") != "GO":
    raise SystemExit("expected local full-runtime live policy final_decision=GO")
if summary_report.get("three_node_role_set_status") != "verified":
    raise SystemExit("expected summary three_node_role_set_status=verified")
if summary_report.get("transport_propagation_status") != "verified":
    raise SystemExit("expected summary transport_propagation_status=verified")
if summary_report.get("canonical_convergence_status") != "verified":
    raise SystemExit("expected summary canonical_convergence_status=verified")
if summary_report.get("runtime_transport_mode") != "libp2p_transport_fed":
    raise SystemExit("expected summary runtime_transport_mode=libp2p_transport_fed")

lane_report = {
    "schema_version": "kamn.runtime.local-full-runtime-live-contract-lane-report.v1",
    "status": "pass",
    "final_decision": "GO",
    "lane_mode": mode,
    "local_full_runtime_contract_status": "verified",
    "local_full_runtime_policy_status": policy_report.get("local_full_runtime_policy_status"),
    "docs_contract_status": "verified",
    "three_node_convergence_status": "verified",
    "runtime_transport_mode_status": "verified",
    "fail_closed_status": "verified",
    "fail_closed_reason_code": "local_full_runtime_policy_fast_gate_exclusion_mismatch",
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
echo "local_full_runtime_contract_status=verified"
echo "local_full_runtime_policy_status=verified"
echo "docs_contract_status=verified"
echo "three_node_convergence_status=verified"
echo "runtime_transport_mode_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=local_full_runtime_policy_fast_gate_exclusion_mismatch"
echo "performance_budget_status=verified"
