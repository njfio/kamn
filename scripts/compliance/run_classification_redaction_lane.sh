#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CLASSIFICATION_CONTRACT="$ROOT_DIR/scripts/compliance/run_dsar_legal_hold_contract_lane.sh"
REDACTION_CONTRACT="$ROOT_DIR/scripts/channel/run_channel_retention_redaction_contract_lane.sh"
CLASSIFICATION_DOC="$ROOT_DIR/docs/foundation/data-classification-tagging.md"
REDACTION_DOC="$ROOT_DIR/docs/foundation/redaction-tombstones.md"

usage() {
  cat <<'EOF'
Usage:
  bash scripts/compliance/run_classification_redaction_lane.sh \
    [--output-file <path>]
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

output_file="$ROOT_DIR/classification-redaction-report.json"

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

max_runtime_seconds="${KAMN_CLASSIFICATION_REDACTION_MAX_SECONDS:-180}"
if [[ ! "$max_runtime_seconds" =~ ^[0-9]+$ ]]; then
  fail "KAMN_CLASSIFICATION_REDACTION_MAX_SECONDS must be an integer >= 0"
fi

skip_commands="${KAMN_CLASSIFICATION_REDACTION_SKIP_COMMANDS:-false}"
case "$skip_commands" in
  true|false) ;;
  *)
    fail "KAMN_CLASSIFICATION_REDACTION_SKIP_COMMANDS must be true or false"
    ;;
esac

lane_failed=false
classification_contract_present=true
redaction_contract_present=true
docs_contract_present=true
commands=()

if [[ ! -x "$CLASSIFICATION_CONTRACT" ]]; then
  classification_contract_present=false
fi
if [[ ! -x "$REDACTION_CONTRACT" ]]; then
  redaction_contract_present=false
fi

if [[ "${KAMN_CLASSIFICATION_REDACTION_FORCE_CLASSIFICATION_MISSING:-false}" == "true" ]]; then
  classification_contract_present=false
fi
if [[ "${KAMN_CLASSIFICATION_REDACTION_FORCE_REDACTION_MISSING:-false}" == "true" ]]; then
  redaction_contract_present=false
fi

start_epoch="$(date +%s)"

if [[ "$skip_commands" != "true" ]]; then
  if [[ "$classification_contract_present" == "true" ]]; then
    commands+=("bash scripts/compliance/run_dsar_legal_hold_contract_lane.sh")
    if ! bash "$CLASSIFICATION_CONTRACT" >/dev/null; then
      lane_failed=true
    fi
  fi

  if [[ "$redaction_contract_present" == "true" ]]; then
    commands+=("bash scripts/channel/run_channel_retention_redaction_contract_lane.sh --skip-tests")
    if ! bash "$REDACTION_CONTRACT" --skip-tests >/dev/null; then
      lane_failed=true
    fi
  fi
fi

if [[ "${KAMN_CLASSIFICATION_REDACTION_FORCE_LANE_FAILURE:-false}" == "true" ]]; then
  lane_failed=true
fi

required_doc_markers=(
  "run_classification_redaction_contract_lane.sh"
  "check_classification_redaction_policy.sh"
  "kamn.compliance.classification-redaction-report.v1"
  "classification_redaction_reason_codes:GO:v1"
  "classification_redaction_reason_codes:NO-GO:v1"
  'classification/redaction contract drift must fail closed (`Regression: #914`).'
)

for marker in "${required_doc_markers[@]}"; do
  if ! grep -Fq "$marker" "$CLASSIFICATION_DOC"; then
    docs_contract_present=false
  fi
  if ! grep -Fq "$marker" "$REDACTION_DOC"; then
    docs_contract_present=false
  fi
done

if [[ "${KAMN_CLASSIFICATION_REDACTION_FORCE_DOCS_CONTRACT_MISSING:-false}" == "true" ]]; then
  docs_contract_present=false
fi

runtime_seconds="$(( $(date +%s) - start_epoch ))"
runtime_budget_ok=true
if [ "$runtime_seconds" -gt "$max_runtime_seconds" ]; then
  runtime_budget_ok=false
fi

decision_reasons=()
if [[ "$lane_failed" == "true" ]]; then
  decision_reasons+=("classification_redaction_lane_failed")
fi
if [[ "$classification_contract_present" != "true" ]]; then
  decision_reasons+=("classification_contract_missing")
fi
if [[ "$redaction_contract_present" != "true" ]]; then
  decision_reasons+=("redaction_contract_missing")
fi
if [[ "$docs_contract_present" != "true" ]]; then
  decision_reasons+=("docs_contract_missing")
fi
if [[ "$runtime_budget_ok" != "true" ]]; then
  decision_reasons+=("runtime_budget_exceeded")
fi

final_decision="GO"
if [ "${#decision_reasons[@]}" -gt 0 ]; then
  final_decision="NO-GO"
fi
reason_key="classification_redaction_reason_codes:${final_decision}:v1"

mkdir -p "$(dirname "$output_file")"

decision_reasons_json="$(python3 -c 'import json,sys; print(json.dumps(sys.argv[1:]))' "${decision_reasons[@]}")"
commands_json="$(python3 -c 'import json,sys; print(json.dumps(sys.argv[1:]))' "${commands[@]}")"

python3 - "$output_file" "$max_runtime_seconds" "$runtime_seconds" "$decision_reasons_json" "$commands_json" "$lane_failed" "$classification_contract_present" "$redaction_contract_present" "$docs_contract_present" "$runtime_budget_ok" "$final_decision" "$reason_key" <<'PY'
import json
import pathlib
import sys
from datetime import datetime, timezone

output_file = pathlib.Path(sys.argv[1])
max_runtime_seconds = int(sys.argv[2])
runtime_seconds = int(sys.argv[3])
decision_reasons = json.loads(sys.argv[4])
commands = json.loads(sys.argv[5])
lane_failed = sys.argv[6] == "true"
classification_contract_present = sys.argv[7] == "true"
redaction_contract_present = sys.argv[8] == "true"
docs_contract_present = sys.argv[9] == "true"
runtime_budget_ok = sys.argv[10] == "true"
final_decision = sys.argv[11]
reason_key = sys.argv[12]

payload = {
    "schema_version": "kamn.compliance.classification-redaction-report.v1",
    "generated_at": datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "max_runtime_seconds": max_runtime_seconds,
    "runtime_seconds": runtime_seconds,
    "checks": {
        "lane_failed": lane_failed,
        "classification_contract_present": classification_contract_present,
        "redaction_contract_present": redaction_contract_present,
        "docs_contract_present": docs_contract_present,
        "runtime_budget_ok": runtime_budget_ok,
    },
    "commands": commands,
    "decision_reasons": decision_reasons,
    "final_decision": final_decision,
    "reason_key": reason_key,
}

output_file.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

printf 'status=ok\n'
printf 'output_file=%s\n' "$output_file"
printf 'final_decision=%s\n' "$final_decision"
printf 'reason_key=%s\n' "$reason_key"
printf 'runtime_seconds=%s\n' "$runtime_seconds"
printf 'max_runtime_seconds=%s\n' "$max_runtime_seconds"
