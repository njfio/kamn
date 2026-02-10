#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/signer/run_signer_incident_recovery_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'USAGE'
Usage:
  KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE=scheduled \
    bash scripts/signer/run_signer_incident_recovery_deep_lane.sh \
      [--output-json <path>] \
      [--report-file <path>] \
      [--skip-contract-lane]
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

require_positive_int_env() {
  local name="$1"
  local value="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]] || [ "$value" -le 0 ]; then
    fail "${name} must be a positive integer"
  fi
}

output_json="$ROOT_DIR/signer-incident-recovery-deep-report.json"
report_file=""
skip_contract_lane=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --report-file)
      report_file="${2:-}"
      shift 2
      ;;
    --skip-contract-lane)
      skip_contract_lane=true
      shift
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

if [ ! -x "$CONTRACT_LANE" ]; then
  fail "expected signer incident recovery contract lane script to be executable"
fi

cadence="${KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE:-}"
if [ "$cadence" != "scheduled" ]; then
  fail "scheduled-only cadence policy requires KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE=scheduled"
fi

max_seconds="${KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_MAX_SECONDS:-240}"
max_artifact_age_seconds="${KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_MAX_ARTIFACT_AGE_SECONDS:-3600}"
force_stale_artifact="${KAMN_SIGNER_INCIDENT_RECOVERY_FORCE_STALE_ARTIFACT:-false}"
require_positive_int_env "KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_MAX_SECONDS" "$max_seconds"
require_positive_int_env "KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_MAX_ARTIFACT_AGE_SECONDS" "$max_artifact_age_seconds"

if [ "$force_stale_artifact" != "true" ] && [ "$force_stale_artifact" != "false" ]; then
  fail "KAMN_SIGNER_INCIDENT_RECOVERY_FORCE_STALE_ARTIFACT must be true or false"
fi

mkdir -p "$(dirname "$output_json")"
start_epoch="$(date +%s)"
contract_lane_invoked=false

if [ "$skip_contract_lane" != true ]; then
  contract_lane_invoked=true
  if [ -z "$report_file" ]; then
    report_file="$TMP_DIR/signer-incident-recovery-contract-report.json"
  fi
  bash "$CONTRACT_LANE" --output-file "$report_file" >/dev/null
fi

if [ -z "$report_file" ]; then
  fail "--report-file is required when --skip-contract-lane is used"
fi

if [ ! -f "$report_file" ]; then
  fail "report file not found: $report_file"
fi

report_meta="$(
  python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
generated_epoch = payload.get("generated_epoch")
final_decision = payload.get("final_decision")
schema_version = payload.get("schema_version")
if not isinstance(generated_epoch, int):
    raise SystemExit("generated_epoch must be an integer in signer incident recovery report")
if not isinstance(final_decision, str):
    raise SystemExit("final_decision must be a string in signer incident recovery report")
if not isinstance(schema_version, str):
    raise SystemExit("schema_version must be a string in signer incident recovery report")
print(f"generated_epoch={generated_epoch}")
print(f"final_decision={final_decision}")
print(f"schema_version={schema_version}")
PY
)"

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

source_generated_epoch="$(extract_value "$report_meta" "generated_epoch")"
source_final_decision="$(extract_value "$report_meta" "final_decision")"
source_schema_version="$(extract_value "$report_meta" "schema_version")"

if [[ -z "$source_generated_epoch" ]] || [[ ! "$source_generated_epoch" =~ ^[0-9]+$ ]]; then
  fail "unable to parse generated_epoch from signer incident recovery report"
fi

artifact_age_seconds="$(( $(date +%s) - source_generated_epoch ))"
if [ "$force_stale_artifact" = "true" ]; then
  artifact_age_seconds="$(( max_artifact_age_seconds + 1 ))"
fi

reason_codes=()
if [ "$source_final_decision" != "GO" ]; then
  reason_codes+=("source_report_not_go")
fi
if [ "$artifact_age_seconds" -gt "$max_artifact_age_seconds" ]; then
  reason_codes+=("stale_deep_artifact")
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  reason_codes+=("runtime_budget_exceeded")
fi

if [ "${#reason_codes[@]}" -gt 0 ]; then
  IFS=$'\n' read -r -d '' -a reason_codes < <(printf '%s\n' "${reason_codes[@]}" | sort -u && printf '\0')
fi

status="pass"
final_decision="GO"
if [ "${#reason_codes[@]}" -gt 0 ]; then
  status="fail"
  final_decision="NO-GO"
fi

reason_codes_csv="none"
if [ "${#reason_codes[@]}" -gt 0 ]; then
  reason_codes_csv="$(IFS=,; echo "${reason_codes[*]}")"
fi
reason_key="signer_incident_recovery_deep_reason_codes:${final_decision}:v1"

python3 - "$output_json" "$status" "$final_decision" "$reason_key" "$report_file" "$source_schema_version" "$source_final_decision" "$artifact_age_seconds" "$max_artifact_age_seconds" "$elapsed_seconds" "$max_seconds" "$reason_codes_csv" "$contract_lane_invoked" <<'PY'
import json
import pathlib
import sys

(
    output_json,
    status,
    final_decision,
    reason_key,
    report_file,
    source_schema_version,
    source_final_decision,
    artifact_age_seconds,
    max_artifact_age_seconds,
    elapsed_seconds,
    max_seconds,
    reason_codes_csv,
    contract_lane_invoked,
) = sys.argv[1:]

reason_codes = [] if reason_codes_csv == "none" else reason_codes_csv.split(",")

payload = {
    "schema_version": "kamn.signer.incident-recovery-deep-summary.v1",
    "lane": "deep",
    "cadence": "scheduled",
    "status": status,
    "final_decision": final_decision,
    "reason_key": reason_key,
    "source_report_file": report_file,
    "source_report_schema": source_schema_version,
    "source_report_final_decision": source_final_decision,
    "artifact_age_seconds": int(artifact_age_seconds),
    "max_artifact_age_seconds": int(max_artifact_age_seconds),
    "elapsed_seconds": int(elapsed_seconds),
    "max_seconds": int(max_seconds),
    "reason_codes": reason_codes,
    "stale_artifact_blocked": "stale_deep_artifact" in reason_codes,
    "contract_lane_invoked": contract_lane_invoked == "true",
}

pathlib.Path(output_json).write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
PY

if [ "$status" != "pass" ]; then
  fail "signer incident recovery deep lane failed closed: ${reason_codes_csv}"
fi

echo "signer incident recovery deep lane tests passed."
