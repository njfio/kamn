#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/sdk/run_live_transport_smoke_parity_lane.sh \
    [--output-json <path>] \
    [--languages <rust,python,typescript>]
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

normalize_languages() {
  local raw="$1"
  local token normalized=()
  local seen_rust=false
  local seen_python=false
  local seen_typescript=false

  if [ -z "$raw" ] || [ "$raw" = "all" ]; then
    printf 'rust,python,typescript\n'
    return 0
  fi

  IFS=',' read -r -a tokens <<<"$raw"
  for token in "${tokens[@]}"; do
    case "$(printf '%s' "$token" | tr '[:upper:]' '[:lower:]' | xargs)" in
      rust)
        if [ "$seen_rust" = false ]; then
          normalized+=("rust")
          seen_rust=true
        fi
        ;;
      python)
        if [ "$seen_python" = false ]; then
          normalized+=("python")
          seen_python=true
        fi
        ;;
      typescript)
        if [ "$seen_typescript" = false ]; then
          normalized+=("typescript")
          seen_typescript=true
        fi
        ;;
      "")
        ;;
      *)
        fail "unsupported language selector: $token"
        ;;
    esac
  done

  if [ "${#normalized[@]}" -eq 0 ]; then
    fail "at least one language must be selected"
  fi

  printf '%s,' "${normalized[@]}" | sed 's/,$//'
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PARITY_RUNNER="$ROOT_DIR/scripts/sdk/run_live_transport_parity_contract_lane.sh"
OUTPUT_JSON=""
LANGUAGES="${KAMN_SDK_SMOKE_PARITY_LANGUAGES:-rust,python,typescript}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      OUTPUT_JSON="${2:-}"
      shift 2
      ;;
    --languages)
      LANGUAGES="${2:-}"
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

if [ ! -x "$PARITY_RUNNER" ]; then
  fail "expected live transport parity contract lane runner to be executable"
fi

SELECTED_LANGUAGES="$(normalize_languages "$LANGUAGES")"
MAX_SECONDS="${KAMN_SDK_SMOKE_PARITY_MAX_SECONDS:-180}"
MAX_RETRIES="${KAMN_SDK_SMOKE_PARITY_MAX_RETRIES:-1}"
FAKE_DELAY_SECONDS="${KAMN_SDK_SMOKE_PARITY_FAKE_DELAY_SECONDS:-0}"
SKIP_COMMANDS="${KAMN_SDK_SMOKE_PARITY_SKIP_COMMANDS:-false}"
FORCE_FAILURE="${KAMN_SDK_SMOKE_PARITY_FORCE_FAILURE:-false}"

if [[ ! "$MAX_SECONDS" =~ ^[0-9]+$ ]]; then
  fail "KAMN_SDK_SMOKE_PARITY_MAX_SECONDS must be a non-negative integer"
fi

if [[ ! "$MAX_RETRIES" =~ ^[0-9]+$ ]] || [ "$MAX_RETRIES" -gt 2 ]; then
  fail "KAMN_SDK_SMOKE_PARITY_MAX_RETRIES must be an integer between 0 and 2"
fi

if [[ ! "$FAKE_DELAY_SECONDS" =~ ^[0-9]+$ ]]; then
  fail "KAMN_SDK_SMOKE_PARITY_FAKE_DELAY_SECONDS must be a non-negative integer"
fi

if [[ "$SKIP_COMMANDS" != "true" && "$SKIP_COMMANDS" != "false" ]]; then
  fail "KAMN_SDK_SMOKE_PARITY_SKIP_COMMANDS must be true or false"
fi

if [[ "$FORCE_FAILURE" != "true" && "$FORCE_FAILURE" != "false" ]]; then
  fail "KAMN_SDK_SMOKE_PARITY_FORCE_FAILURE must be true or false"
fi

cd "$ROOT_DIR"
start_epoch="$(date +%s)"

if [ "$FAKE_DELAY_SECONDS" -gt 0 ]; then
  sleep "$FAKE_DELAY_SECONDS"
fi

commands=()
run_smoke_attempt() {
  if [[ "$FORCE_FAILURE" == "true" ]]; then
    return 1
  fi

  if [[ "$SKIP_COMMANDS" == "true" ]]; then
    return 0
  fi

  commands+=("bash scripts/sdk/run_live_transport_parity_contract_lane.sh --languages ${SELECTED_LANGUAGES}")
  bash "$PARITY_RUNNER" --languages "$SELECTED_LANGUAGES" >/dev/null
}

max_attempts="$((MAX_RETRIES + 1))"
attempt=1
retry_used=false
retry_final_status="failed"

while true; do
  if run_smoke_attempt; then
    retry_final_status="passed"
    break
  fi

  if [ "$attempt" -ge "$max_attempts" ]; then
    break
  fi

  retry_used=true
  attempt="$((attempt + 1))"
done

retry_attempts="$attempt"
if [ "$retry_attempts" -gt 1 ]; then
  retry_used=true
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"

reason_codes=()
if [[ "$retry_final_status" != "passed" ]]; then
  reason_codes+=("smoke_lane_failed")
fi
if [[ "$retry_final_status" != "passed" && "$retry_used" == "true" ]]; then
  reason_codes+=("retry_budget_exceeded")
fi
if [ "$elapsed_seconds" -gt "$MAX_SECONDS" ]; then
  reason_codes+=("runtime_budget_exceeded")
fi

if [ "${#reason_codes[@]}" -gt 0 ]; then
  mapfile -t reason_codes < <(printf '%s\n' "${reason_codes[@]}" | sort)
fi

status="pass"
final_decision="GO"
if [ "${#reason_codes[@]}" -gt 0 ]; then
  status="fail"
  final_decision="NO-GO"
fi

reason_codes_csv="none"
if [ "${#reason_codes[@]}" -gt 0 ]; then
  reason_codes_csv="$(printf '%s,' "${reason_codes[@]}" | sed 's/,$//')"
fi

if [[ -n "$OUTPUT_JSON" ]]; then
  mkdir -p "$(dirname "$OUTPUT_JSON")"
  python3 - "$OUTPUT_JSON" "$status" "$final_decision" "$elapsed_seconds" "$MAX_SECONDS" "$MAX_RETRIES" "$retry_attempts" "$retry_used" "$retry_final_status" "$SELECTED_LANGUAGES" "$SKIP_COMMANDS" "$FORCE_FAILURE" "$reason_codes_csv" "${commands[@]}" <<'PY'
import json
import pathlib
import sys

output_file = pathlib.Path(sys.argv[1])
status = sys.argv[2]
final_decision = sys.argv[3]
elapsed_seconds = int(sys.argv[4])
max_seconds = int(sys.argv[5])
max_retries = int(sys.argv[6])
retry_attempts = int(sys.argv[7])
retry_used = sys.argv[8] == "true"
retry_final_status = sys.argv[9]
languages = [item for item in sys.argv[10].split(",") if item]
skip_commands = sys.argv[11] == "true"
force_failure = sys.argv[12] == "true"
reason_codes_csv = sys.argv[13]
commands = sys.argv[14:]

payload = {
    "schema_version": "kamn.sdk.live-transport-smoke-parity-report.v1",
    "status": status,
    "final_decision": final_decision,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "max_retries": max_retries,
    "retry_attempts": retry_attempts,
    "retry_used": retry_used,
    "retry_final_status": retry_final_status,
    "languages": languages,
    "skip_commands": skip_commands,
    "force_failure": force_failure,
    "command_count": len(commands),
    "commands": commands,
    "reason_codes": [] if reason_codes_csv == "none" else reason_codes_csv.split(","),
}
output_file.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY
fi

printf 'status=%s\n' "$status"
printf 'final_decision=%s\n' "$final_decision"
printf 'elapsed_seconds=%s\n' "$elapsed_seconds"
printf 'max_seconds=%s\n' "$MAX_SECONDS"
printf 'retry_attempts=%s\n' "$retry_attempts"
printf 'max_retries=%s\n' "$MAX_RETRIES"
printf 'retry_used=%s\n' "$retry_used"
printf 'retry_final_status=%s\n' "$retry_final_status"
printf 'failed_checks=%s\n' "$reason_codes_csv"
if [[ -n "$OUTPUT_JSON" ]]; then
  printf 'report_file=%s\n' "$(cd "$(dirname "$OUTPUT_JSON")" && pwd)/$(basename "$OUTPUT_JSON")"
fi

if [[ "$status" != "pass" ]]; then
  fail "sdk live transport smoke parity lane failed closed: ${reason_codes_csv}"
fi

echo "sdk live transport smoke parity lane tests passed."
