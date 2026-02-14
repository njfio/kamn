#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-runtime-commit-live-summary.json"
LIVE_OUTPUT_FILE="/tmp/kolme-local-runtime-commit-live-output.txt"
LIVE_COMMAND=""
FINALITY_COMMAND=""
MAX_SECONDS=90
BASE_URL="http://127.0.0.1:3000"
PROVIDER_HINT="kolme-fork-local"
AUTHORIZATION_HEADER=""
PREFLIGHT_MAX_SECONDS=10
FINALITY_MAX_SECONDS=15
FINALITY_RETRY_MAX_ATTEMPTS=1
FINALITY_RETRY_BACKOFF_SECONDS=1
SKIP_PREFLIGHT=0
FINALITY_OUTPUT_FILE="/tmp/kolme-local-runtime-commit-live-finality-output.txt"


shell_escape() {
  printf "%q" "$1"
}

default_live_command() {
  local command
  command="KAMN_KOLME_LIVE_BASE_URL=$(shell_escape "$BASE_URL")"
  command="${command} KAMN_KOLME_LIVE_PROVIDER_HINT=$(shell_escape "$PROVIDER_HINT")"
  command="${command} KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1"
  if [ -n "$AUTHORIZATION_HEADER" ]; then
    command="${command} KAMN_KOLME_LIVE_AUTHORIZATION=$(shell_escape "$AUTHORIZATION_HEADER")"
  fi
  command="${command} cargo test -p kamn-core --test kolme_runtime_commit_http_transport -- --exact integration_kolme_fork_live_node_submit_reaches_endpoint && printf 'replay_guard=verified\\n'"
  printf '%s' "$command"
}

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
    --live-output-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --live-output-file" >&2
        exit 1
      fi
      LIVE_OUTPUT_FILE="$2"
      shift 2
      ;;
    --live-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --live-command" >&2
        exit 1
      fi
      LIVE_COMMAND="$2"
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
    --finality-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --finality-command" >&2
        exit 1
      fi
      FINALITY_COMMAND="$2"
      shift 2
      ;;
    --finality-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --finality-max-seconds" >&2
        exit 1
      fi
      FINALITY_MAX_SECONDS="$2"
      shift 2
      ;;
    --finality-retry-max-attempts)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --finality-retry-max-attempts" >&2
        exit 1
      fi
      FINALITY_RETRY_MAX_ATTEMPTS="$2"
      shift 2
      ;;
    --finality-retry-backoff-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --finality-retry-backoff-seconds" >&2
        exit 1
      fi
      FINALITY_RETRY_BACKOFF_SECONDS="$2"
      shift 2
      ;;
    --finality-output-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --finality-output-file" >&2
        exit 1
      fi
      FINALITY_OUTPUT_FILE="$2"
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
    --provider-hint)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --provider-hint" >&2
        exit 1
      fi
      PROVIDER_HINT="$2"
      shift 2
      ;;
    --authorization-header)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --authorization-header" >&2
        exit 1
      fi
      AUTHORIZATION_HEADER="$2"
      shift 2
      ;;
    --preflight-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --preflight-max-seconds" >&2
        exit 1
      fi
      PREFLIGHT_MAX_SECONDS="$2"
      shift 2
      ;;
    --skip-preflight)
      SKIP_PREFLIGHT=1
      shift
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_runtime_commit_live_lane.sh [options]

Options:
  --mode dry-run|run            Execute planned output or active local live lane.
  --output-json <path>          Deterministic summary report output path.
  --live-output-file <path>     Captured stdout/stderr for live command.
  --live-command <command>      Runtime-commit submit/finality command for run mode.
  --finality-command <command>  Optional post-submit finality command for run mode.
  --finality-output-file <path> Captured stdout/stderr for finality command.
  --finality-max-seconds <n>    Max runtime budget in seconds for finality command.
  --finality-retry-max-attempts <n>
                                Max bounded retry attempts for finality command in run mode.
  --finality-retry-backoff-seconds <n>
                                Backoff wait in seconds between finality retry attempts.
  --max-seconds <n>             Max runtime budget in seconds for run mode.
  --base-url <url>              Live Kolme base URL used by default smoke command.
  --provider-hint <value>       Provider hint used by default live smoke command.
  --authorization-header <str>  Optional Authorization header value for live smoke.
  --preflight-max-seconds <n>   Max runtime budget for preflight health probe.
  --skip-preflight              Bypass preflight health probe in run mode.
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

if ! [[ "$MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$MAX_SECONDS" -le 0 ]; then
  echo "max-seconds must be a positive integer" >&2
  exit 1
fi

if ! [[ "$PREFLIGHT_MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$PREFLIGHT_MAX_SECONDS" -le 0 ]; then
  echo "preflight-max-seconds must be a positive integer" >&2
  exit 1
fi

if ! [[ "$FINALITY_MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$FINALITY_MAX_SECONDS" -le 0 ]; then
  echo "finality-max-seconds must be a positive integer" >&2
  exit 1
fi

if ! [[ "$FINALITY_RETRY_MAX_ATTEMPTS" =~ ^[0-9]+$ ]] || [ "$FINALITY_RETRY_MAX_ATTEMPTS" -le 0 ]; then
  echo "finality-retry-max-attempts must be a positive integer" >&2
  exit 1
fi

if ! [[ "$FINALITY_RETRY_BACKOFF_SECONDS" =~ ^[0-9]+$ ]]; then
  echo "finality-retry-backoff-seconds must be a non-negative integer" >&2
  exit 1
fi

if [ -z "$BASE_URL" ]; then
  echo "base-url must not be empty" >&2
  exit 1
fi

if [ -z "$PROVIDER_HINT" ]; then
  echo "provider-hint must not be empty" >&2
  exit 1
fi

if [[ "$PROVIDER_HINT" == *"InMemoryKolmeRuntimeCommitClient"* ]]; then
  echo "provider-hint must not reference InMemoryKolmeRuntimeCommitClient" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if [ -z "$LIVE_COMMAND" ]; then
  LIVE_COMMAND="$(default_live_command)"
fi

EXPECTED_SIGNING_PROFILE_MARKER="KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1"
if [[ "$LIVE_COMMAND" != *"$EXPECTED_SIGNING_PROFILE_MARKER"* ]]; then
  echo "live-command must set KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1" >&2
  exit 1
fi

if [[ "$LIVE_COMMAND" == *"KAMN_KOLME_LIVE_SIGNING_PROFILE=simulated"* ]]; then
  echo "live-command must not reference simulated signing profiles" >&2
  exit 1
fi

if [[ "$LIVE_COMMAND" == *"InMemoryKolmeRuntimeCommitClient"* ]]; then
  echo "live-command must not reference InMemoryKolmeRuntimeCommitClient" >&2
  exit 1
fi

PROVIDER_CLIENT_CONTRACT="KolmeRuntimeCommitLiveProvider"
PROVIDER_SUBMIT_PROFILE_CONTRACT="kolme_fork_broadcast_profile"
PROVIDER_COMMAND_MARKER="integration_kolme_fork_live_node_submit_reaches_endpoint"
PROVIDER_COMMAND_MARKER_PRESENT="false"
if [[ "$LIVE_COMMAND" == *"$PROVIDER_COMMAND_MARKER"* ]]; then
  PROVIDER_COMMAND_MARKER_PRESENT="true"
fi
PROVIDER_SIGNING_PROFILE_MARKER="$EXPECTED_SIGNING_PROFILE_MARKER"
PROVIDER_SIGNING_PROFILE_MARKER_PRESENT="false"
if [[ "$LIVE_COMMAND" == *"$PROVIDER_SIGNING_PROFILE_MARKER"* ]]; then
  PROVIDER_SIGNING_PROFILE_MARKER_PRESENT="true"
fi

SUBMIT_EVIDENCE_MARKER="status=submitted"
SUBMIT_EVIDENCE_MARKER_PRESENT="false"
if [[ "$LIVE_COMMAND" == *"$SUBMIT_EVIDENCE_MARKER"* ]]; then
  SUBMIT_EVIDENCE_MARKER_PRESENT="true"
fi

FINALITY_EVIDENCE_MARKER="finality=final"
FINALITY_EVIDENCE_MARKER_PRESENT="false"
if [ -n "$FINALITY_COMMAND" ] && [[ "$FINALITY_COMMAND" == *"$FINALITY_EVIDENCE_MARKER"* ]]; then
  FINALITY_EVIDENCE_MARKER_PRESENT="true"
fi

NATIVE_PAYLOAD_PUBKEY_MARKER='"pubkey"'
NATIVE_PAYLOAD_PUBKEY_MARKER_PRESENT="false"
if [[ "$LIVE_COMMAND" == *"$NATIVE_PAYLOAD_PUBKEY_MARKER"* ]]; then
  NATIVE_PAYLOAD_PUBKEY_MARKER_PRESENT="true"
fi

NATIVE_PAYLOAD_NONCE_MARKER='"nonce"'
NATIVE_PAYLOAD_NONCE_MARKER_PRESENT="false"
if [[ "$LIVE_COMMAND" == *"$NATIVE_PAYLOAD_NONCE_MARKER"* ]]; then
  NATIVE_PAYLOAD_NONCE_MARKER_PRESENT="true"
fi

NATIVE_PAYLOAD_MESSAGES_MARKER='"messages"'
NATIVE_PAYLOAD_MESSAGES_MARKER_PRESENT="false"
if [[ "$LIVE_COMMAND" == *"$NATIVE_PAYLOAD_MESSAGES_MARKER"* ]]; then
  NATIVE_PAYLOAD_MESSAGES_MARKER_PRESENT="true"
fi

REQUEST_PAYLOAD_EVIDENCE_MARKER="native_payload_pubkey_nonce_messages"
REQUEST_PAYLOAD_EVIDENCE_MARKER_PRESENT="false"
REQUEST_PAYLOAD_EVIDENCE_ARTIFACT_PATH="$LIVE_OUTPUT_FILE"
SUBMIT_EVIDENCE_ARTIFACT_PATH="$LIVE_OUTPUT_FILE"
FINALITY_EVIDENCE_ARTIFACT_PATH=""
if [ -n "$FINALITY_COMMAND" ]; then
  FINALITY_EVIDENCE_ARTIFACT_PATH="$FINALITY_OUTPUT_FILE"
fi
REQUEST_FINALITY_EVIDENCE_CONTRACT_VERSION="v1"
REQUEST_FINALITY_EVIDENCE_LINKED="false"
FINALITY_RETRY_CONTRACT_VERSION="v1"
FINALITY_RETRY_ATTEMPTS_USED=0
FINALITY_RETRY_EXHAUSTED="false"
FINALITY_RETRY_FAILURE_CLASS="none"

preflight_command="curl --silent --show-error --fail --max-time ${PREFLIGHT_MAX_SECONDS} ${BASE_URL%/}/healthz"

CHECK_FILE="$(mktemp)"
trap 'rm -f "$CHECK_FILE"' EXIT

record_check() {
  local check_id="$1"
  local command="$2"
  local status="$3"
  printf '%s\t%s\t%s\n' "$check_id" "$command" "$status" >>"$CHECK_FILE"
}

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0
local_only_enforced="true"

planned_command="$LIVE_COMMAND"
planned_finality_command="$FINALITY_COMMAND"
if [ -z "$planned_finality_command" ]; then
  planned_finality_command="<not-configured>"
fi
record_check "runtime_commit_live_preflight" "$preflight_command" "planned"
record_check "runtime_commit_live_command" "$planned_command" "planned"
record_check "runtime_commit_live_finality_command" "$planned_finality_command" "planned"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if ! "$LOCAL_HEAVY_GUARD"; then
    record_check "runtime_commit_live_command" "$planned_command" "fail"
    record_check "runtime_commit_live_finality_command" "$planned_finality_command" "skipped"
    overall_status="fail"
    reason_code="local_opt_in_missing"
  elif [ "$SKIP_PREFLIGHT" -eq 1 ]; then
    record_check "runtime_commit_live_preflight" "$preflight_command" "skipped"
  else
    set +e
    timeout "${PREFLIGHT_MAX_SECONDS}" bash -lc "$preflight_command" >/dev/null 2>&1
    preflight_exit_code=$?
    set -e

    if [ "$preflight_exit_code" -eq 0 ]; then
      record_check "runtime_commit_live_preflight" "$preflight_command" "pass"
    elif [ "$preflight_exit_code" -eq 124 ]; then
      record_check "runtime_commit_live_preflight" "$preflight_command" "fail"
      overall_status="fail"
      reason_code="live_preflight_timeout"
      record_check "runtime_commit_live_finality_command" "$planned_finality_command" "skipped"
    else
      record_check "runtime_commit_live_preflight" "$preflight_command" "fail"
      overall_status="fail"
      reason_code="live_preflight_failed"
      record_check "runtime_commit_live_finality_command" "$planned_finality_command" "skipped"
    fi
  fi

  if [ "$overall_status" = "ok" ]; then
    mkdir -p "$(dirname "$LIVE_OUTPUT_FILE")"
    command_exit_code=0
    set +e
    timeout "${MAX_SECONDS}" bash -lc "$LIVE_COMMAND" >"$LIVE_OUTPUT_FILE" 2>&1
    command_exit_code=$?
    set -e

    if [ "$command_exit_code" -eq 0 ]; then
      record_check "runtime_commit_live_command" "$LIVE_COMMAND" "pass"
      reason_code="live_runtime_commit_command_passed"
    elif [ "$command_exit_code" -eq 124 ]; then
      record_check "runtime_commit_live_command" "$LIVE_COMMAND" "fail"
      overall_status="fail"
      reason_code="live_runtime_commit_command_timeout"
      record_check "runtime_commit_live_finality_command" "$planned_finality_command" "skipped"
    else
      record_check "runtime_commit_live_command" "$LIVE_COMMAND" "fail"
      overall_status="fail"
      reason_code="live_runtime_commit_command_failed"
      record_check "runtime_commit_live_finality_command" "$planned_finality_command" "skipped"
    fi
  fi

  if [ "$overall_status" = "ok" ]; then
    if [ -n "$FINALITY_COMMAND" ]; then
      mkdir -p "$(dirname "$FINALITY_OUTPUT_FILE")"
      finality_succeeded=0
      finality_attempt=0
      finality_check_status="fail"
      while [ "$finality_attempt" -lt "$FINALITY_RETRY_MAX_ATTEMPTS" ]; do
        finality_attempt=$((finality_attempt + 1))
        set +e
        timeout "${FINALITY_MAX_SECONDS}" bash -lc "$FINALITY_COMMAND" >"$FINALITY_OUTPUT_FILE" 2>&1
        finality_exit_code=$?
        set -e

        FINALITY_RETRY_ATTEMPTS_USED="$finality_attempt"
        if [ "$finality_exit_code" -eq 0 ]; then
          finality_succeeded=1
          finality_check_status="pass"
          FINALITY_RETRY_EXHAUSTED="false"
          FINALITY_RETRY_FAILURE_CLASS="none"
          record_check "runtime_commit_live_finality_command_attempt_${finality_attempt}" "$FINALITY_COMMAND" "pass"
          reason_code="live_runtime_commit_and_finality_commands_passed"
          break
        fi

        if [ "$finality_exit_code" -eq 124 ]; then
          FINALITY_RETRY_FAILURE_CLASS="timeout"
        else
          FINALITY_RETRY_FAILURE_CLASS="failed"
        fi
        record_check "runtime_commit_live_finality_command_attempt_${finality_attempt}" "$FINALITY_COMMAND" "fail"

        if [ "$finality_attempt" -lt "$FINALITY_RETRY_MAX_ATTEMPTS" ] && [ "$FINALITY_RETRY_BACKOFF_SECONDS" -gt 0 ]; then
          sleep "$FINALITY_RETRY_BACKOFF_SECONDS"
        fi
      done

      if [ "$finality_succeeded" -eq 1 ]; then
        record_check "runtime_commit_live_finality_command" "$FINALITY_COMMAND" "$finality_check_status"
      else
        record_check "runtime_commit_live_finality_command" "$FINALITY_COMMAND" "fail"
        FINALITY_RETRY_EXHAUSTED="true"
        overall_status="fail"
        if [ "$FINALITY_RETRY_FAILURE_CLASS" = "timeout" ]; then
          reason_code="live_finality_retry_exhausted_timeout"
        else
          reason_code="live_finality_retry_exhausted_failed"
        fi
      fi
    else
      record_check "runtime_commit_live_finality_command" "$planned_finality_command" "skipped"
      FINALITY_RETRY_ATTEMPTS_USED=0
      FINALITY_RETRY_EXHAUSTED="false"
      FINALITY_RETRY_FAILURE_CLASS="none"
    fi
  fi

  end_epoch="$(date +%s)"
  elapsed_seconds="$(( end_epoch - start_epoch ))"
  if [ "$elapsed_seconds" -le "$MAX_SECONDS" ] && [ "$reason_code" != "live_runtime_commit_command_timeout" ] && [ "$reason_code" != "live_finality_retry_exhausted_timeout" ]; then
    budget_status="within_budget"
  else
    budget_status="exceeded_budget"
    if [ "$overall_status" = "ok" ]; then
      overall_status="fail"
      reason_code="live_runtime_commit_budget_exceeded"
    fi
  fi
fi

if [ "$MODE" = "run" ]; then
  if [ -f "$LIVE_OUTPUT_FILE" ] && grep -Fq "$SUBMIT_EVIDENCE_MARKER" "$LIVE_OUTPUT_FILE"; then
    SUBMIT_EVIDENCE_MARKER_PRESENT="true"
  else
    SUBMIT_EVIDENCE_MARKER_PRESENT="false"
  fi

  if [ -n "$FINALITY_COMMAND" ] && [ -f "$FINALITY_OUTPUT_FILE" ] && grep -Fq "$FINALITY_EVIDENCE_MARKER" "$FINALITY_OUTPUT_FILE"; then
    FINALITY_EVIDENCE_MARKER_PRESENT="true"
  else
    FINALITY_EVIDENCE_MARKER_PRESENT="false"
  fi

  if [ -f "$LIVE_OUTPUT_FILE" ] && grep -Fq "$NATIVE_PAYLOAD_PUBKEY_MARKER" "$LIVE_OUTPUT_FILE"; then
    NATIVE_PAYLOAD_PUBKEY_MARKER_PRESENT="true"
  else
    NATIVE_PAYLOAD_PUBKEY_MARKER_PRESENT="false"
  fi

  if [ -f "$LIVE_OUTPUT_FILE" ] && grep -Fq "$NATIVE_PAYLOAD_NONCE_MARKER" "$LIVE_OUTPUT_FILE"; then
    NATIVE_PAYLOAD_NONCE_MARKER_PRESENT="true"
  else
    NATIVE_PAYLOAD_NONCE_MARKER_PRESENT="false"
  fi

  if [ -f "$LIVE_OUTPUT_FILE" ] && grep -Fq "$NATIVE_PAYLOAD_MESSAGES_MARKER" "$LIVE_OUTPUT_FILE"; then
    NATIVE_PAYLOAD_MESSAGES_MARKER_PRESENT="true"
  else
    NATIVE_PAYLOAD_MESSAGES_MARKER_PRESENT="false"
  fi
fi

if [ "$NATIVE_PAYLOAD_PUBKEY_MARKER_PRESENT" = "true" ] \
  && [ "$NATIVE_PAYLOAD_NONCE_MARKER_PRESENT" = "true" ] \
  && [ "$NATIVE_PAYLOAD_MESSAGES_MARKER_PRESENT" = "true" ]; then
  REQUEST_PAYLOAD_EVIDENCE_MARKER_PRESENT="true"
else
  REQUEST_PAYLOAD_EVIDENCE_MARKER_PRESENT="false"
fi

if [ "$MODE" = "run" ] \
  && [ -n "$FINALITY_COMMAND" ] \
  && [ "$REQUEST_PAYLOAD_EVIDENCE_MARKER_PRESENT" = "true" ] \
  && [ "$SUBMIT_EVIDENCE_MARKER_PRESENT" = "true" ] \
  && [ "$FINALITY_EVIDENCE_MARKER_PRESENT" = "true" ]; then
  REQUEST_FINALITY_EVIDENCE_LINKED="true"
else
  REQUEST_FINALITY_EVIDENCE_LINKED="false"
fi

PROVIDER_CONTRACT_ENFORCEMENT_MODE="live-provider-only-v1"
PROVIDER_LIVE_CONTRACT_MARKER="provider_client_contract=${PROVIDER_CLIENT_CONTRACT}"
PROVIDER_LIVE_CONTRACT_MARKER_PRESENT="false"
if [ "$PROVIDER_CLIENT_CONTRACT" = "KolmeRuntimeCommitLiveProvider" ]; then
  PROVIDER_LIVE_CONTRACT_MARKER_PRESENT="true"
fi
PROVIDER_IN_MEMORY_REFERENCE_DETECTED="false"
if [[ "$PROVIDER_HINT" == *"InMemoryKolmeRuntimeCommitClient"* ]] || [[ "$LIVE_COMMAND" == *"InMemoryKolmeRuntimeCommitClient"* ]]; then
  PROVIDER_IN_MEMORY_REFERENCE_DETECTED="true"
fi
PROVIDER_SIGNER_ADAPTER_CONTRACT="KolmeForkSecp256k1SignerAdapter"
PROVIDER_SIGNING_CURVE_CONTRACT="secp256k1"
PROVIDER_SIGNING_PROFILE_CONTRACT_VERSION="v1"

python3 "$ROOT_DIR/scripts/kolme/contracts/local_runtime_commit_live_summary.py" "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$local_only_enforced" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$LIVE_COMMAND" "$LIVE_OUTPUT_FILE" "$FINALITY_COMMAND" "$FINALITY_OUTPUT_FILE" "$BASE_URL" "$PROVIDER_HINT" "$PREFLIGHT_MAX_SECONDS" "$FINALITY_MAX_SECONDS" "$SKIP_PREFLIGHT" "$CHECK_FILE" "$PROVIDER_CLIENT_CONTRACT" "$PROVIDER_SUBMIT_PROFILE_CONTRACT" "$PROVIDER_COMMAND_MARKER" "$PROVIDER_COMMAND_MARKER_PRESENT" "$PROVIDER_SIGNING_PROFILE_MARKER" "$PROVIDER_SIGNING_PROFILE_MARKER_PRESENT" "$SUBMIT_EVIDENCE_MARKER" "$SUBMIT_EVIDENCE_MARKER_PRESENT" "$FINALITY_EVIDENCE_MARKER" "$FINALITY_EVIDENCE_MARKER_PRESENT" "$NATIVE_PAYLOAD_PUBKEY_MARKER" "$NATIVE_PAYLOAD_PUBKEY_MARKER_PRESENT" "$NATIVE_PAYLOAD_NONCE_MARKER" "$NATIVE_PAYLOAD_NONCE_MARKER_PRESENT" "$NATIVE_PAYLOAD_MESSAGES_MARKER" "$NATIVE_PAYLOAD_MESSAGES_MARKER_PRESENT" "$REQUEST_PAYLOAD_EVIDENCE_MARKER" "$REQUEST_PAYLOAD_EVIDENCE_MARKER_PRESENT" "$REQUEST_PAYLOAD_EVIDENCE_ARTIFACT_PATH" "$SUBMIT_EVIDENCE_ARTIFACT_PATH" "$FINALITY_EVIDENCE_ARTIFACT_PATH" "$REQUEST_FINALITY_EVIDENCE_CONTRACT_VERSION" "$REQUEST_FINALITY_EVIDENCE_LINKED" "$FINALITY_RETRY_CONTRACT_VERSION" "$FINALITY_RETRY_MAX_ATTEMPTS" "$FINALITY_RETRY_BACKOFF_SECONDS" "$FINALITY_RETRY_ATTEMPTS_USED" "$FINALITY_RETRY_EXHAUSTED" "$FINALITY_RETRY_FAILURE_CLASS" "$PROVIDER_CONTRACT_ENFORCEMENT_MODE" "$PROVIDER_LIVE_CONTRACT_MARKER" "$PROVIDER_LIVE_CONTRACT_MARKER_PRESENT" "$PROVIDER_IN_MEMORY_REFERENCE_DETECTED" "$PROVIDER_SIGNER_ADAPTER_CONTRACT" "$PROVIDER_SIGNING_CURVE_CONTRACT" "$PROVIDER_SIGNING_PROFILE_CONTRACT_VERSION"

echo "status=$overall_status"
echo "lane_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "local_only_enforced=$local_only_enforced"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
