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
  --fast-workflow-file <path>     Fast-gate workflow file for rerun-policy checks.
  --deep-workflow-file <path>     Deep-validate workflow file for rerun-policy checks.
  --output-json <path>            Output JSON policy report.
  -h, --help                      Show this help.
EOF
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

registry_file=".ci/flaky-tests.txt"
max_active_entries=0
expected_final_decision="GO"
fast_workflow_file=".github/workflows/ci-fast-gate.yml"
deep_workflow_file=".github/workflows/ci-deep-validate.yml"
output_json="/tmp/anti-flake-policy-report.json"
reason_taxonomy_version="kamn.ci.anti-flake-policy-reason-taxonomy.v1"

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
    --fast-workflow-file)
      fast_workflow_file="${2:-}"
      shift 2
      ;;
    --deep-workflow-file)
      deep_workflow_file="${2:-}"
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

if [ ! -f "$fast_workflow_file" ]; then
  reason_codes+=("rerun_policy_fast_workflow_missing")
  status="fail"
  final_decision="NO-GO"
else
  if ! grep -q -- '--max-attempts 2' "$fast_workflow_file"; then
    reason_codes+=("rerun_policy_bounded_retry_missing")
    status="fail"
    final_decision="NO-GO"
  fi
  if ! grep -q -- '--max-attempts 1' "$fast_workflow_file"; then
    reason_codes+=("rerun_policy_invariant_non_retry_missing")
    status="fail"
    final_decision="NO-GO"
  fi
  if grep -q -- '--max-attempts 3' "$fast_workflow_file"; then
    reason_codes+=("rerun_policy_excessive_retry_detected")
    status="fail"
    final_decision="NO-GO"
  fi
fi

if [ ! -f "$deep_workflow_file" ]; then
  reason_codes+=("rerun_policy_deep_workflow_missing")
  status="fail"
  final_decision="NO-GO"
else
  if ! grep -q -- '--max-attempts 2' "$deep_workflow_file"; then
    reason_codes+=("rerun_policy_bounded_retry_missing")
    status="fail"
    final_decision="NO-GO"
  fi
  if ! grep -q -- '--max-attempts 1' "$deep_workflow_file"; then
    reason_codes+=("rerun_policy_invariant_non_retry_missing")
    status="fail"
    final_decision="NO-GO"
  fi
  if grep -q -- '--max-attempts 3' "$deep_workflow_file"; then
    reason_codes+=("rerun_policy_excessive_retry_detected")
    status="fail"
    final_decision="NO-GO"
  fi
fi

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

reason_codes_csv="$(printf '%s\n' "${reason_codes[@]}" | sed '/^$/d' | sort -u | paste -sd, -)"
if [ -z "$reason_codes_csv" ]; then
  reason_codes_csv="policy_evaluation_incomplete"
fi
IFS=',' read -r -a reason_codes <<<"$reason_codes_csv"
reason_codes_value="$reason_codes_csv"

reason_class="violation"
if [[ "$final_decision" = "GO" ]]; then
  if [[ "$reason_codes_csv" = "no_active_flaky_entries" ]]; then
    reason_class="stable"
  elif [[ "$reason_codes_csv" = "active_flaky_entries_within_budget" ]]; then
    reason_class="budgeted"
  else
    reason_class="stable"
  fi
fi

mkdir -p "$(dirname "$output_json")"
python3 - \
  "$output_json" \
  "$status" \
  "$final_decision" \
  "$reason_taxonomy_version" \
  "$reason_codes_csv" \
  "$reason_codes_value" \
  "$reason_class" \
  "$registry_file" \
  "$fast_workflow_file" \
  "$deep_workflow_file" \
  "$active_entries" \
  "$max_active_entries" \
  "$expected_final_decision" <<'PY'
import json
import pathlib
import sys

output_json = pathlib.Path(sys.argv[1])
status = sys.argv[2]
final_decision = sys.argv[3]
reason_taxonomy_version = sys.argv[4]
reason_codes_csv = sys.argv[5]
reason_codes_value = sys.argv[6]
reason_class = sys.argv[7]
registry_file = sys.argv[8]
fast_workflow_file = sys.argv[9]
deep_workflow_file = sys.argv[10]
active_entries = int(sys.argv[11])
max_active_entries = int(sys.argv[12])
expected_final_decision = sys.argv[13]

reason_codes = [code for code in reason_codes_csv.split(",") if code]
payload = {
    "schema_version": "kamn.ci.anti-flake-policy-report.v1",
    "reason_taxonomy_version": reason_taxonomy_version,
    "status": status,
    "final_decision": final_decision,
    "reason_codes": reason_codes,
    "reason_codes_csv": reason_codes_csv,
    "reason_codes_value": reason_codes_value,
    "reason_class": reason_class,
    "registry_file": registry_file,
    "fast_workflow_file": fast_workflow_file,
    "deep_workflow_file": deep_workflow_file,
    "active_entries": active_entries,
    "max_active_entries": max_active_entries,
    "expected_final_decision": expected_final_decision,
}
output_json.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "anti_flake_policy_status=$status"
echo "anti_flake_policy_final_decision=$final_decision"
echo "anti_flake_policy_reason_taxonomy_version=$reason_taxonomy_version"
echo "anti_flake_policy_reason_codes=$reason_codes_csv"
echo "anti_flake_policy_reason_codes_csv=$reason_codes_csv"
echo "anti_flake_policy_reason_codes_value=$reason_codes_value"
echo "anti_flake_policy_reason_class=$reason_class"
echo "anti_flake_policy_active_entries=$active_entries"
echo "anti_flake_policy_max_active_entries=$max_active_entries"
echo "anti_flake_policy_registry_file=$registry_file"
echo "anti_flake_policy_report_file=$output_json"

if [[ "$final_decision" != "GO" ]]; then
  exit 1
fi

exit 0
