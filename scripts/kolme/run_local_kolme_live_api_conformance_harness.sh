#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PROBE_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_api_probe_lane.sh"
NATIVE_RUNNER="$ROOT_DIR/scripts/kolme/run_local_native_api_parity_live_proof_lane.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-live-api-conformance-summary.json"
PROBE_REPORT="/tmp/kolme-local-api-probe-summary.json"
NATIVE_REPORT="/tmp/kolme-local-native-api-parity-live-proof-summary.json"
BASE_URL="http://127.0.0.1:3000"
FORK_CHAIN_VERSION="v0.15.2"
NONCE_PUBKEY="test-key"
BROADCAST_PAYLOAD='{"message":"native-parity","signature":"sig","recovery_id":1}'
MAX_SECONDS=180
PROBE_MAX_SECONDS=30
NATIVE_MAX_SECONDS=120

while [ "$#" -gt 0 ]; do
  case "$1" in
    --mode)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --mode" >&2
        exit 1
      fi
      MODE="$2"
      shift 2
      ;;
    --output-json)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --output-json" >&2
        exit 1
      fi
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --probe-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --probe-report" >&2
        exit 1
      fi
      PROBE_REPORT="$2"
      shift 2
      ;;
    --native-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --native-report" >&2
        exit 1
      fi
      NATIVE_REPORT="$2"
      shift 2
      ;;
    --base-url)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --base-url" >&2
        exit 1
      fi
      BASE_URL="$2"
      shift 2
      ;;
    --fork-chain-version)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --fork-chain-version" >&2
        exit 1
      fi
      FORK_CHAIN_VERSION="$2"
      shift 2
      ;;
    --nonce-pubkey)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --nonce-pubkey" >&2
        exit 1
      fi
      NONCE_PUBKEY="$2"
      shift 2
      ;;
    --broadcast-payload)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --broadcast-payload" >&2
        exit 1
      fi
      BROADCAST_PAYLOAD="$2"
      shift 2
      ;;
    --max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --max-seconds" >&2
        exit 1
      fi
      MAX_SECONDS="$2"
      shift 2
      ;;
    --probe-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --probe-max-seconds" >&2
        exit 1
      fi
      PROBE_MAX_SECONDS="$2"
      shift 2
      ;;
    --native-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --native-max-seconds" >&2
        exit 1
      fi
      NATIVE_MAX_SECONDS="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_live_api_conformance_harness.sh [options]

Options:
  --mode dry-run|run              Emit planned checks or execute local live conformance checks.
  --output-json <path>            Deterministic summary report output path.
  --probe-report <path>           Output path for local API probe summary.
  --native-report <path>          Output path for local native parity summary.
  --base-url <url>                Base URL for local Kolme API server.
  --fork-chain-version <value>    Required chain_version query value for fork-info checks.
  --nonce-pubkey <value>          Pubkey query value for /get-next-nonce.
  --broadcast-payload <json>      JSON payload sent to PUT /broadcast.
  --max-seconds <n>               Max total runtime budget for the harness.
  --probe-max-seconds <n>         Max runtime for probe prerequisite.
  --native-max-seconds <n>        Max runtime for native parity proof step.
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [ "$MODE" != "dry-run" ] && [ "$MODE" != "run" ]; then
  echo "mode must be one of: dry-run, run" >&2
  exit 1
fi

if [ -z "$BASE_URL" ] || [ -z "$FORK_CHAIN_VERSION" ] || [ -z "$NONCE_PUBKEY" ]; then
  echo "base-url, fork-chain-version, and nonce-pubkey must not be empty" >&2
  exit 1
fi

if [ -z "$BROADCAST_PAYLOAD" ]; then
  echo "broadcast-payload must not be empty" >&2
  exit 1
fi

if ! [[ "$MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$MAX_SECONDS" -le 0 ]; then
  echo "max-seconds must be a positive integer" >&2
  exit 1
fi

if ! [[ "$PROBE_MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$PROBE_MAX_SECONDS" -le 0 ]; then
  echo "probe-max-seconds must be a positive integer" >&2
  exit 1
fi

if ! [[ "$NATIVE_MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$NATIVE_MAX_SECONDS" -le 0 ]; then
  echo "native-max-seconds must be a positive integer" >&2
  exit 1
fi

if [ ! -x "$PROBE_RUNNER" ]; then
  echo "expected local Kolme API probe runner to be executable" >&2
  exit 1
fi

if [ ! -x "$NATIVE_RUNNER" ]; then
  echo "expected local native API parity live proof runner to be executable" >&2
  exit 1
fi

CHECK_FILE="$(mktemp)"
trap 'rm -f "$CHECK_FILE"' EXIT

record_check() {
  local check_id="$1"
  local command="$2"
  local status="$3"
  local reason_code="$4"
  printf '%s\t%s\t%s\t%s\n' "$check_id" "$command" "$status" "$reason_code" >>"$CHECK_FILE"
}

read_report_reason_code() {
  local report_file="$1"
  python3 - "$report_file" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
if not path.exists():
    print("report_missing")
    raise SystemExit(0)

try:
    payload = json.loads(path.read_text(encoding="utf-8"))
except json.JSONDecodeError:
    print("report_invalid_json")
    raise SystemExit(0)

value = payload.get("reason_code")
if isinstance(value, str) and value.strip():
    print(value)
else:
    print("reason_code_missing")
PY
}

nonce_url="${BASE_URL%/}/get-next-nonce?pubkey=${NONCE_PUBKEY}"
broadcast_url="${BASE_URL%/}/broadcast"
finality_url="${BASE_URL%/}/healthz"

nonce_command="curl --silent --show-error --fail ${nonce_url}"
broadcast_command="curl --silent --show-error --fail --request PUT --header \"Content-Type: application/json\" --data '${BROADCAST_PAYLOAD}' ${broadcast_url}"
finality_command="curl --silent --show-error --fail ${finality_url}"

probe_command="bash scripts/kolme/run_local_kolme_api_probe_lane.sh --mode run --base-url ${BASE_URL} --fork-chain-version ${FORK_CHAIN_VERSION} --max-seconds ${PROBE_MAX_SECONDS} --output-json ${PROBE_REPORT}"
native_command="bash scripts/kolme/run_local_native_api_parity_live_proof_lane.sh --mode run --nonce-command \"${nonce_command}\" --broadcast-command \"${broadcast_command}\" --finality-command \"${finality_command}\" --max-seconds ${NATIVE_MAX_SECONDS} --output-json ${NATIVE_REPORT}"

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0
probe_reason_code="not_run"
native_reason_code="not_run"

record_check "api_probe" "$probe_command" "planned" "not_run"
record_check "native_api_parity" "$native_command" "planned" "not_run"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if [ "${KAMN_KOLME_LOCAL_HEAVY:-0}" != "1" ]; then
    echo "run mode requires explicit local-only opt-in: KAMN_KOLME_LOCAL_HEAVY=1" >&2
    record_check "api_probe" "$probe_command" "fail" "local_opt_in_missing"
    record_check "native_api_parity" "$native_command" "skipped" "local_opt_in_missing"
    overall_status="fail"
    reason_code="local_opt_in_missing"
    probe_reason_code="local_opt_in_missing"
    native_reason_code="local_opt_in_missing"
  else
    if bash "$PROBE_RUNNER" \
      --mode run \
      --base-url "$BASE_URL" \
      --fork-chain-version "$FORK_CHAIN_VERSION" \
      --max-seconds "$PROBE_MAX_SECONDS" \
      --output-json "$PROBE_REPORT" >/dev/null; then
      record_check "api_probe" "$probe_command" "pass" "probe_checks_passed"
      probe_reason_code="probe_checks_passed"
    else
      probe_reason_code="$(read_report_reason_code "$PROBE_REPORT")"
      record_check "api_probe" "$probe_command" "fail" "$probe_reason_code"
      record_check "native_api_parity" "$native_command" "skipped" "probe_prerequisite_failed"
      overall_status="fail"
      reason_code="probe_conformance_failed"
      native_reason_code="probe_prerequisite_failed"
    fi

    if [ "$overall_status" = "ok" ]; then
      if KAMN_KOLME_LOCAL_HEAVY=1 bash "$NATIVE_RUNNER" \
        --mode run \
        --nonce-command "$nonce_command" \
        --broadcast-command "$broadcast_command" \
        --finality-command "$finality_command" \
        --max-seconds "$NATIVE_MAX_SECONDS" \
        --output-json "$NATIVE_REPORT" >/dev/null; then
        record_check "native_api_parity" "$native_command" "pass" "native_parity_live_proof_passed"
        native_reason_code="native_parity_live_proof_passed"
        reason_code="live_api_conformance_passed"
      else
        native_reason_code="$(read_report_reason_code "$NATIVE_REPORT")"
        record_check "native_api_parity" "$native_command" "fail" "$native_reason_code"
        overall_status="fail"
        reason_code="native_conformance_failed"
      fi
    fi
  fi

  elapsed_seconds="$(( $(date +%s) - start_epoch ))"
  if [ "$elapsed_seconds" -le "$MAX_SECONDS" ]; then
    budget_status="within_budget"
  else
    budget_status="exceeded_budget"
    if [ "$overall_status" = "ok" ]; then
      overall_status="fail"
      reason_code="harness_budget_exceeded"
    fi
  fi
fi

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$BASE_URL" "$FORK_CHAIN_VERSION" "$NONCE_PUBKEY" "$BROADCAST_PAYLOAD" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$PROBE_REPORT" "$NATIVE_REPORT" "$probe_reason_code" "$native_reason_code" "$CHECK_FILE" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
base_url = sys.argv[5]
fork_chain_version = sys.argv[6]
nonce_pubkey = sys.argv[7]
broadcast_payload = sys.argv[8]
elapsed_seconds = int(sys.argv[9])
max_seconds = int(sys.argv[10])
budget_status = sys.argv[11]
probe_report = sys.argv[12]
native_report = sys.argv[13]
probe_reason_code = sys.argv[14]
native_reason_code = sys.argv[15]
checks_path = pathlib.Path(sys.argv[16])

checks = []
for raw_line in checks_path.read_text(encoding="utf-8").splitlines():
    if not raw_line.strip():
        continue
    parts = raw_line.split("\t")
    if len(parts) != 4:
        continue
    check_id, command, check_status, check_reason = parts
    checks.append(
        {
            "id": check_id,
            "command": command,
            "status": check_status,
            "reason_code": check_reason,
        }
    )

summary = {
    "schema_version": "kamn.kolme.local-live-api-conformance-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "base_url": base_url,
    "fork_chain_version": fork_chain_version,
    "nonce_pubkey": nonce_pubkey,
    "probe_reason_code": probe_reason_code,
    "native_reason_code": native_reason_code,
    "checks": checks,
    "contracts": {
        "healthz_path": "/healthz",
        "fork_info_path": "/fork-info",
        "fork_info_query_key": "chain_version",
        "nonce_endpoint": "/get-next-nonce",
        "broadcast_endpoint": "/broadcast",
        "broadcast_method": "PUT",
        "broadcast_payload": broadcast_payload,
    },
    "artifact_paths": [
        probe_report,
        native_report,
    ],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "status=$overall_status"
echo "harness_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "local_only_enforced=true"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
