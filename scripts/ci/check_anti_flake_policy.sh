#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: check_anti_flake_policy.sh [options]

Evaluates anti-flake merge policy against the flaky quarantine registry.

Options:
  --registry-file <path>          Flaky registry file path.
  --max-active-entries <int>      Maximum allowed active quarantine entries.
  --expected-final-decision <GO|NO-GO>
                                  Expected final decision marker.
  --output-json <path>            Output JSON policy report.
  -h, --help                      Show this help.
EOF
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

registry_file=".ci/flaky-tests.txt"
max_active_entries=0
expected_final_decision="GO"
output_json="/tmp/anti-flake-policy-report.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --registry-file)
      registry_file="${2:-}"
      shift 2
      ;;
    --max-active-entries)
      max_active_entries="${2:-}"
      shift 2
      ;;
    --expected-final-decision)
      expected_final_decision="${2:-}"
      shift 2
      ;;
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! [[ "$max_active_entries" =~ ^[0-9]+$ ]]; then
  echo "max-active-entries must be a non-negative integer" >&2
  exit 1
fi

if [[ "$expected_final_decision" != "GO" && "$expected_final_decision" != "NO-GO" ]]; then
  echo "expected-final-decision must be GO or NO-GO" >&2
  exit 1
fi

reason_codes=()
status="pass"
final_decision="GO"

if [ ! -f "$registry_file" ]; then
  reason_codes+=("registry_file_missing")
  status="fail"
  final_decision="NO-GO"
else
  if ! bash "$script_dir/check_flaky_registry.sh" "$registry_file" >/dev/null 2>&1; then
    reason_codes+=("registry_validation_failed")
    status="fail"
    final_decision="NO-GO"
  fi
fi

active_entries=0
if [ -f "$registry_file" ]; then
  while IFS= read -r line || [ -n "$line" ]; do
    case "$line" in
      ""|\#*)
        continue
        ;;
    esac
    active_entries=$((active_entries + 1))
  done < "$registry_file"
fi

if [[ "$final_decision" = "GO" ]]; then
  if [ "$active_entries" -eq 0 ]; then
    reason_codes+=("no_active_flaky_entries")
  elif [ "$active_entries" -le "$max_active_entries" ]; then
    reason_codes+=("active_flaky_entries_within_budget")
  else
    reason_codes+=("active_flaky_entries_exceed_max")
    status="fail"
    final_decision="NO-GO"
  fi
fi

if [[ "$final_decision" != "$expected_final_decision" ]]; then
  reason_codes=("expected_final_decision_mismatch")
  status="fail"
  final_decision="NO-GO"
fi

if [ "${#reason_codes[@]}" -eq 0 ]; then
  reason_codes=("policy_evaluation_incomplete")
  status="fail"
  final_decision="NO-GO"
fi

mkdir -p "$(dirname "$output_json")"

reason_codes_csv="$(IFS=,; echo "${reason_codes[*]}")"
python3 - \
  "$output_json" \
  "$status" \
  "$final_decision" \
  "$reason_codes_csv" \
  "$registry_file" \
  "$active_entries" \
  "$max_active_entries" \
  "$expected_final_decision" <<'PY'
import json
import pathlib
import sys

output_json = pathlib.Path(sys.argv[1])
status = sys.argv[2]
final_decision = sys.argv[3]
reason_codes_csv = sys.argv[4]
registry_file = sys.argv[5]
active_entries = int(sys.argv[6])
max_active_entries = int(sys.argv[7])
expected_final_decision = sys.argv[8]

reason_codes = [code for code in reason_codes_csv.split(",") if code]
payload = {
    "schema_version": "kamn.ci.anti-flake-policy-report.v1",
    "status": status,
    "final_decision": final_decision,
    "reason_codes": reason_codes,
    "registry_file": registry_file,
    "active_entries": active_entries,
    "max_active_entries": max_active_entries,
    "expected_final_decision": expected_final_decision,
}
output_json.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "anti_flake_policy_status=$status"
echo "anti_flake_policy_final_decision=$final_decision"
echo "anti_flake_policy_reason_codes=$reason_codes_csv"
echo "anti_flake_policy_active_entries=$active_entries"
echo "anti_flake_policy_max_active_entries=$max_active_entries"
echo "anti_flake_policy_registry_file=$registry_file"
echo "anti_flake_policy_report_file=$output_json"

if [[ "$final_decision" != "GO" ]]; then
  exit 1
fi

exit 0
