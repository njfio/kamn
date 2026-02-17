#!/usr/bin/env bash
set -euo pipefail

require_env_var() {
  local var_name="$1"
  if [[ -z "${!var_name:-}" ]]; then
    echo "missing required lane configuration variable: $var_name" >&2
    exit 1
  fi
}

service_api_contract_lane_run() {
  require_env_var "ROOT_DIR"
  require_env_var "LANE_LABEL"
  require_env_var "LANE_SLUG"
  require_env_var "VALIDATION_SCRIPT"
  require_env_var "POLICY_CHECKER"
  require_env_var "STRATEGY_DOC"
  require_env_var "ROADMAP_DOC"
  require_env_var "MAX_SECONDS_ENV"
  require_env_var "MAX_SECONDS_DEFAULT"
  require_env_var "CONTRACT_STATUS_KEY"
  require_env_var "POLICY_STATUS_KEY"
  require_env_var "SUMMARY_SCHEMA"
  require_env_var "POLICY_SCHEMA"
  require_env_var "LANE_REPORT_SCHEMA"
  require_env_var "TAMPER_FIELD"
  require_env_var "TAMPER_REASON_CODE"
  require_env_var "ROADMAP_TASK_MARKER"
  require_env_var "ROADMAP_CONTRACT_SCRIPT_REF"
  require_env_var "ROADMAP_POLICY_SCRIPT_REF"

  local output_json=""
  local policy_output_json=""
  local max_seconds
  local ci_fast_gate="PASS"
  local allow_mode="${ALLOW_MODE:-0}"
  local mode="${DEFAULT_MODE:-dry-run}"
  local runbook_doc="${RUNBOOK_DOC:-}"
  local runbook_taxonomy_drift_reason_code="${RUNBOOK_TAXONOMY_DRIFT_REASON_CODE:-protocol_taxonomy_mapping_drift_detected}"
  local runbook_marker_parity_reason_code="${RUNBOOK_MARKER_PARITY_REASON_CODE:-runbook_marker_parity_mismatch}"
  local runbook_required_marker_count=0
  if [[ "${RUNBOOK_REQUIRED_MARKERS+set}" == "set" ]]; then
    runbook_required_marker_count="${#RUNBOOK_REQUIRED_MARKERS[@]}"
  fi
  if (( runbook_required_marker_count > 0 )) && [[ -z "$runbook_doc" ]]; then
    echo "missing required lane configuration variable: RUNBOOK_DOC" >&2
    exit 1
  fi

  # shellcheck disable=SC2086
  max_seconds="${!MAX_SECONDS_ENV:-$MAX_SECONDS_DEFAULT}"

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
        if [[ "$allow_mode" != "1" ]]; then
          echo "unknown argument: --mode" >&2
          exit 1
        fi
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
  if (( max_seconds <= 0 )); then
    echo "max-seconds must be greater than zero" >&2
    exit 1
  fi
  if [[ "$ci_fast_gate" != "PASS" && "$ci_fast_gate" != "FAIL" ]]; then
    echo "ci-fast-gate must be PASS or FAIL" >&2
    exit 1
  fi
  if [[ "$allow_mode" == "1" && "$mode" != "dry-run" && "$mode" != "run" ]]; then
    echo "mode must be dry-run or run" >&2
    exit 1
  fi

  local required_exec
  for required_exec in "$VALIDATION_SCRIPT" "$POLICY_CHECKER"; do
    if [[ ! -x "$required_exec" ]]; then
      echo "expected required executable script '$required_exec'" >&2
      exit 1
    fi
  done

  local required_doc
  for required_doc in "$STRATEGY_DOC" "$ROADMAP_DOC"; do
    if [[ ! -f "$required_doc" ]]; then
      echo "expected required documentation file '$required_doc'" >&2
      exit 1
    fi
  done
  if (( runbook_required_marker_count > 0 )) && [[ ! -f "$runbook_doc" ]]; then
    echo "${runbook_marker_parity_reason_code}: expected required documentation file '$runbook_doc'" >&2
    exit 1
  fi

  local start_epoch
  start_epoch="$(date +%s)"
  local tmp_dir
  tmp_dir="$(mktemp -d)"
  trap "rm -rf '$tmp_dir'" EXIT

  local summary_report="$tmp_dir/${LANE_SLUG}-summary.json"
  local policy_report="$tmp_dir/${LANE_SLUG}-policy.json"
  local tampered_report="$tmp_dir/${LANE_SLUG}-summary.tampered.json"

  local -a validation_cmd=(bash "$VALIDATION_SCRIPT" --output-json "$summary_report" --max-seconds "$max_seconds")
  if [[ "$allow_mode" == "1" ]]; then
    validation_cmd+=(--mode "$mode")
  fi

  local validation_output
  validation_output="$("${validation_cmd[@]}")"

  local marker
  for marker in "${VALIDATION_REQUIRED_MARKERS[@]}"; do
    if ! printf '%s\n' "$validation_output" | grep -q "^${marker}$"; then
      echo "expected ${LANE_LABEL} validation marker ${marker}" >&2
      exit 1
    fi
  done

  local regex_marker
  for regex_marker in "${VALIDATION_REQUIRED_REGEX_MARKERS[@]}"; do
    if ! printf '%s\n' "$validation_output" | grep -Eq "$regex_marker"; then
      echo "expected ${LANE_LABEL} validation regex marker ${regex_marker}" >&2
      exit 1
    fi
  done

  local policy_output
  policy_output="$(
    bash "$POLICY_CHECKER" \
      --report-file "$summary_report" \
      --expected-final-decision GO \
      --ci-fast-gate "$ci_fast_gate" \
      --output-json "$policy_report"
  )"

  for marker in "${POLICY_REQUIRED_MARKERS[@]}"; do
    if ! printf '%s\n' "$policy_output" | grep -q "^${marker}$"; then
      echo "expected ${LANE_LABEL} policy marker ${marker}" >&2
      exit 1
    fi
  done

  cp "$summary_report" "$tampered_report"
  python3 - "$tampered_report" "$TAMPER_FIELD" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
field = sys.argv[2]
payload = json.loads(path.read_text(encoding="utf-8"))
payload[field] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

  set +e
  local tampered_policy_output
  tampered_policy_output="$(
    bash "$POLICY_CHECKER" \
      --report-file "$tampered_report" \
      --expected-final-decision GO \
      --ci-fast-gate "$ci_fast_gate" \
      --output-json "$tmp_dir/${LANE_SLUG}-policy.tampered.json" 2>&1
  )"
  local tampered_policy_code=$?
  set -e

  if [[ "$tampered_policy_code" -eq 0 ]]; then
    echo "expected tampered ${LANE_LABEL} report to fail policy validation" >&2
    exit 1
  fi
  if ! printf '%s\n' "$tampered_policy_output" | grep -q "$TAMPER_REASON_CODE"; then
    echo "expected deterministic fail-closed reason for tampered ${LANE_LABEL} report" >&2
    exit 1
  fi

  local required_ref
  for required_ref in "${STRATEGY_REQUIRED_REFS[@]}"; do
    if ! grep -q "$required_ref" "$STRATEGY_DOC"; then
      echo "expected CI strategy docs to reference ${required_ref}" >&2
      exit 1
    fi
  done

  local required_marker
  for required_marker in "${STRATEGY_REQUIRED_MARKERS[@]}"; do
    if ! grep -q "$required_marker" "$STRATEGY_DOC"; then
      echo "expected CI strategy docs to include marker: ${required_marker}" >&2
      exit 1
    fi
  done

  if ! grep -q "$ROADMAP_TASK_MARKER" "$ROADMAP_DOC"; then
    echo "expected roadmap marker ${ROADMAP_TASK_MARKER}" >&2
    exit 1
  fi
  if ! grep -q "$ROADMAP_CONTRACT_SCRIPT_REF" "$ROADMAP_DOC"; then
    echo "expected roadmap to reference ${ROADMAP_CONTRACT_SCRIPT_REF}" >&2
    exit 1
  fi
  if ! grep -q "$ROADMAP_POLICY_SCRIPT_REF" "$ROADMAP_DOC"; then
    echo "expected roadmap to reference ${ROADMAP_POLICY_SCRIPT_REF}" >&2
    exit 1
  fi

  if (( runbook_required_marker_count > 0 )); then
    for required_marker in "${RUNBOOK_REQUIRED_MARKERS[@]}"; do
      if ! grep -Fq "$required_marker" "$runbook_doc"; then
        local runbook_reason_code="$runbook_marker_parity_reason_code"
        if [[ "$required_marker" == *"_reason_taxonomy_version="* ]] \
          || [[ "$required_marker" == *"_reason_codes_csv="* ]]; then
          runbook_reason_code="$runbook_taxonomy_drift_reason_code"
        fi
        echo "${runbook_reason_code}: missing runbook marker ${required_marker}" >&2
        exit 1
      fi
    done
  fi

  local elapsed_seconds
  elapsed_seconds="$(( $(date +%s) - start_epoch ))"
  if (( elapsed_seconds > max_seconds )); then
    echo "${LANE_LABEL} contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
    exit 1
  fi

  local lane_report="$tmp_dir/${LANE_SLUG}-contract-lane-report.json"
  python3 - \
    "$summary_report" \
    "$policy_report" \
    "$lane_report" \
    "$elapsed_seconds" \
    "$max_seconds" \
    "$SUMMARY_SCHEMA" \
    "$POLICY_SCHEMA" \
    "$LANE_REPORT_SCHEMA" \
    "$CONTRACT_STATUS_KEY" \
    "$POLICY_STATUS_KEY" \
    "$TAMPER_REASON_CODE" \
    "${LANE_REPORT_SUMMARY_FIELDS[@]}" <<'PY'
import json
import pathlib
import sys

summary_report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy_report = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
lane_report_file = pathlib.Path(sys.argv[3])
elapsed_seconds = int(sys.argv[4])
max_seconds = int(sys.argv[5])
summary_schema = sys.argv[6]
policy_schema = sys.argv[7]
lane_report_schema = sys.argv[8]
contract_status_key = sys.argv[9]
policy_status_key = sys.argv[10]
tamper_reason = sys.argv[11]
summary_fields = sys.argv[12:]

if summary_report.get("schema_version") != summary_schema:
    raise SystemExit(f"unexpected summary schema: {summary_report.get('schema_version')}")
if policy_report.get("schema_version") != policy_schema:
    raise SystemExit(f"unexpected policy schema: {policy_report.get('schema_version')}")
if summary_report.get("final_decision") != "GO":
    raise SystemExit("expected summary final_decision=GO")
if policy_report.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")

lane_report = {
    "schema_version": lane_report_schema,
    "status": "pass",
    "final_decision": "GO",
    contract_status_key: "verified",
    policy_status_key: policy_report.get(policy_status_key),
    "docs_contract_status": "verified",
    "fail_closed_status": "verified",
    "fail_closed_reason_code": tamper_reason,
    "performance_budget_status": "verified",
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
}
for field in summary_fields:
    lane_report[field] = summary_report.get(field)

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
  echo "${CONTRACT_STATUS_KEY}=verified"
  echo "${POLICY_STATUS_KEY}=verified"

  if (( ${#OUTPUT_SUMMARY_FIELDS[@]} > 0 )); then
    python3 - "$summary_report" "${OUTPUT_SUMMARY_FIELDS[@]}" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
for key in sys.argv[2:]:
    print(f"{key}={payload.get(key, 0)}")
PY
  fi

  echo "docs_contract_status=verified"
  echo "fail_closed_status=verified"
  echo "fail_closed_reason_code=${TAMPER_REASON_CODE}"
  echo "performance_budget_status=verified"
}
