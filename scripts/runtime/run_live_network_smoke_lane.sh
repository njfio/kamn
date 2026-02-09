#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/runtime/run_live_network_smoke_lane.sh [--output-json <path>]
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUTPUT_JSON=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      OUTPUT_JSON="${2:-}"
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

MAX_SECONDS="${KAMN_LIVE_NETWORK_SMOKE_MAX_SECONDS:-120}"
if [[ ! "$MAX_SECONDS" =~ ^[0-9]+$ ]]; then
  fail "KAMN_LIVE_NETWORK_SMOKE_MAX_SECONDS must be a non-negative integer"
fi

FAKE_DELAY_SECONDS="${KAMN_LIVE_NETWORK_SMOKE_FAKE_DELAY_SECONDS:-0}"
if [[ ! "$FAKE_DELAY_SECONDS" =~ ^[0-9]+$ ]]; then
  fail "KAMN_LIVE_NETWORK_SMOKE_FAKE_DELAY_SECONDS must be a non-negative integer"
fi

SKIP_COMMANDS="${KAMN_LIVE_NETWORK_SMOKE_SKIP_COMMANDS:-false}"
if [[ "$SKIP_COMMANDS" != "true" && "$SKIP_COMMANDS" != "false" ]]; then
  fail "KAMN_LIVE_NETWORK_SMOKE_SKIP_COMMANDS must be true or false"
fi

cd "$ROOT_DIR"
start_epoch="$(date +%s)"

if [[ "$FAKE_DELAY_SECONDS" -gt 0 ]]; then
  sleep "$FAKE_DELAY_SECONDS"
fi

commands=()
if [[ "$SKIP_COMMANDS" != "true" ]]; then
  bash "$ROOT_DIR/scripts/sdk/run_localhost_signed_demo.sh" >/dev/null
  commands+=("scripts/sdk/run_localhost_signed_demo.sh")

  cargo test -p kamn-core --test role_smoke_network functional_roles_complete_smoke_roundtrip_with_gossip -- --exact >/dev/null
  commands+=("cargo_test_role_smoke_network_functional_roundtrip")
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
reason_codes=()
if [[ "$elapsed_seconds" -gt "$MAX_SECONDS" ]]; then
  reason_codes+=("runtime_budget_exceeded")
fi

status="pass"
final_decision="GO"
if [[ "${#reason_codes[@]}" -gt 0 ]]; then
  status="fail"
  final_decision="NO-GO"
fi

reason_codes_csv="none"
if [[ "${#reason_codes[@]}" -gt 0 ]]; then
  reason_codes_csv="$(printf '%s,' "${reason_codes[@]}" | sed 's/,$//')"
fi

if [[ -n "$OUTPUT_JSON" ]]; then
  mkdir -p "$(dirname "$OUTPUT_JSON")"
  python3 - "$OUTPUT_JSON" "$status" "$final_decision" "$elapsed_seconds" "$MAX_SECONDS" "$reason_codes_csv" "$SKIP_COMMANDS" "${commands[@]}" <<'PY'
import json
import pathlib
import sys

output_file = pathlib.Path(sys.argv[1])
status = sys.argv[2]
final_decision = sys.argv[3]
elapsed_seconds = int(sys.argv[4])
max_seconds = int(sys.argv[5])
reason_codes_csv = sys.argv[6]
skip_commands = sys.argv[7] == "true"
commands = sys.argv[8:]

payload = {
    "schema_version": "kamn.runtime.live-network-smoke-report.v1",
    "status": status,
    "final_decision": final_decision,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "skip_commands": skip_commands,
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
printf 'failed_checks=%s\n' "$reason_codes_csv"
if [[ -n "$OUTPUT_JSON" ]]; then
  printf 'report_file=%s\n' "$(cd "$(dirname "$OUTPUT_JSON")" && pwd)/$(basename "$OUTPUT_JSON")"
fi

if [[ "$status" != "pass" ]]; then
  fail "live-network smoke lane exceeded runtime budget: ${elapsed_seconds}s (max=${MAX_SECONDS}s)"
fi

echo "live-network smoke lane tests passed."
