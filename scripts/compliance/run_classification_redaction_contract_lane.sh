#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/compliance/run_classification_redaction_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/compliance/check_classification_redaction_policy.sh"

usage() {
  cat <<'EOF'
Usage:
  bash scripts/compliance/run_classification_redaction_contract_lane.sh \
    [--output-file <path>]
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

if [[ ! -x "$LANE_SCRIPT" ]]; then
  fail "classification/redaction lane script is not executable"
fi
if [[ ! -x "$POLICY_CHECKER" ]]; then
  fail "classification/redaction policy checker script is not executable"
fi

max_runtime="${KAMN_CLASSIFICATION_REDACTION_CONTRACT_MAX_SECONDS:-240}"
if [[ ! "$max_runtime" =~ ^[0-9]+$ ]]; then
  fail "KAMN_CLASSIFICATION_REDACTION_CONTRACT_MAX_SECONDS must be an integer >= 0"
fi
start_epoch="$(date +%s)"

output_file="/tmp/classification-redaction-contract-report.json"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

go_report="$tmp_dir/classification-redaction-go.json"
go_output="$(
  KAMN_CLASSIFICATION_REDACTION_SKIP_COMMANDS=true \
    bash "$LANE_SCRIPT" --output-file "$go_report"
)"
if ! printf '%s\n' "$go_output" | grep -q '^final_decision=GO$'; then
  fail "expected classification/redaction contract lane GO path to produce GO decision"
fi
if ! printf '%s\n' "$go_output" | grep -q '^reason_key=classification_redaction_reason_codes:GO:v1$'; then
  fail "expected classification/redaction contract lane GO path reason_key marker"
fi

go_policy="$(bash "$POLICY_CHECKER" --report-file "$go_report")"
if ! printf '%s\n' "$go_policy" | grep -q '^final_decision=GO$'; then
  fail "expected classification/redaction policy checker GO path decision"
fi

no_go_report="$tmp_dir/classification-redaction-no-go.json"
no_go_output="$(
  KAMN_CLASSIFICATION_REDACTION_SKIP_COMMANDS=true \
  KAMN_CLASSIFICATION_REDACTION_FORCE_DOCS_CONTRACT_MISSING=true \
    bash "$LANE_SCRIPT" --output-file "$no_go_report"
)"
if ! printf '%s\n' "$no_go_output" | grep -q '^final_decision=NO-GO$'; then
  fail "expected classification/redaction contract lane forced docs drift to produce NO-GO"
fi
if ! printf '%s\n' "$no_go_output" | grep -q '^reason_key=classification_redaction_reason_codes:NO-GO:v1$'; then
  fail "expected classification/redaction contract lane NO-GO reason_key marker"
fi

no_go_policy="$(bash "$POLICY_CHECKER" --report-file "$no_go_report")"
if ! printf '%s\n' "$no_go_policy" | grep -q '^final_decision=NO-GO$'; then
  fail "expected classification/redaction policy checker NO-GO path decision"
fi

tampered_report="$tmp_dir/classification-redaction-tampered.json"
cp "$no_go_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["reason_key"] = "classification_redaction_reason_codes:GO:v1"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --report-file "$tampered_report" 2>&1)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  fail "expected classification/redaction reason_key tamper to fail policy checker"
fi
if ! printf '%s\n' "$tampered_output" | grep -q "reason_key mismatch"; then
  fail "expected explicit reason_key mismatch failure from classification/redaction policy checker"
fi

cp "$go_report" "$output_file"

runtime_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$runtime_seconds" -gt "$max_runtime" ]; then
  fail "classification/redaction contract lane exceeded runtime budget (${runtime_seconds}s > ${max_runtime}s)"
fi

printf 'status=ok\n'
printf 'output_file=%s\n' "$output_file"
printf 'final_decision=%s\n' "$(extract_value "$go_output" "final_decision")"
printf 'reason_key=%s\n' "$(extract_value "$go_output" "reason_key")"
echo "classification/redaction compliance contract lane tests passed."
