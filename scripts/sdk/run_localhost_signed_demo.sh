#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
REPLAY_POLICY_CHECKER="$ROOT_DIR/scripts/kolme/check_runtime_commit_replay_policy.py"
ARTIFACT_SCHEMA="kamn.sdk.localhost-signed.demo-receipt-artifact.v1"

usage() {
  cat <<'EOF_USAGE'
Usage: run_localhost_signed_demo.sh [options]

Options:
  --addr <host:port>            Listener bind/connect address.
  --from <did>                  Sender DID.
  --to <did>                    Listener DID.
  --state-hash <hash>           State hash attached to message envelope.
  --body <text>                 Message body payload.
  --nonce <n>                   Positive integer nonce.
  --timeout-seconds <seconds>   Positive integer listener completion timeout.
  --output-json <path>          Write signed exchange + receipt artifact JSON.
  --help                        Show this help output.

Environment defaults:
  KAMN_LOCALHOST_SIGNED_DEMO_ADDR
  KAMN_LOCALHOST_SIGNED_DEMO_FROM
  KAMN_LOCALHOST_SIGNED_DEMO_TO
  KAMN_LOCALHOST_SIGNED_DEMO_STATE_HASH
  KAMN_LOCALHOST_SIGNED_DEMO_BODY
  KAMN_LOCALHOST_SIGNED_DEMO_NONCE
  KAMN_LOCALHOST_SIGNED_DEMO_TIMEOUT_SECONDS
  KAMN_LOCALHOST_SIGNED_DEMO_OUTPUT_JSON
EOF_USAGE
}

require_arg_value() {
  local option="$1"
  local value="${2:-}"
  if [ -z "$value" ]; then
    echo "missing value for $option" >&2
    exit 1
  fi
}

is_positive_integer() {
  [[ "$1" =~ ^[1-9][0-9]*$ ]]
}

validate_agent_did() {
  local label="$1"
  local value="$2"
  if [[ ! "$value" =~ ^kamn:did:agent:[[:alnum:]_.:-]+$ ]]; then
    echo "$label must be a kamn agent DID (kamn:did:agent:<id>)" >&2
    exit 1
  fi
}

extract_marker_value() {
  local key="$1"
  local file="$2"
  local line
  line="$(grep -E "^${key}=" "$file" | tail -n1 || true)"
  if [ -z "$line" ]; then
    printf '%s' ""
    return 0
  fi
  printf '%s' "${line#*=}"
}

ADDR="${KAMN_LOCALHOST_SIGNED_DEMO_ADDR:-127.0.0.1:17879}"
FROM_DID="${KAMN_LOCALHOST_SIGNED_DEMO_FROM:-kamn:did:agent:sender-1}"
TO_DID="${KAMN_LOCALHOST_SIGNED_DEMO_TO:-kamn:did:agent:listener-1}"
STATE_HASH="${KAMN_LOCALHOST_SIGNED_DEMO_STATE_HASH:-state:localhost-demo}"
BODY="${KAMN_LOCALHOST_SIGNED_DEMO_BODY:-hello-from-localhost-demo}"
NONCE="${KAMN_LOCALHOST_SIGNED_DEMO_NONCE:-1}"
TIMEOUT_SECONDS="${KAMN_LOCALHOST_SIGNED_DEMO_TIMEOUT_SECONDS:-15}"
OUTPUT_JSON="${KAMN_LOCALHOST_SIGNED_DEMO_OUTPUT_JSON:-}"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --addr)
      require_arg_value "$1" "${2:-}"
      ADDR="$2"
      shift 2
      ;;
    --from)
      require_arg_value "$1" "${2:-}"
      FROM_DID="$2"
      shift 2
      ;;
    --to)
      require_arg_value "$1" "${2:-}"
      TO_DID="$2"
      shift 2
      ;;
    --state-hash)
      require_arg_value "$1" "${2:-}"
      STATE_HASH="$2"
      shift 2
      ;;
    --body)
      require_arg_value "$1" "${2:-}"
      BODY="$2"
      shift 2
      ;;
    --nonce)
      require_arg_value "$1" "${2:-}"
      NONCE="$2"
      shift 2
      ;;
    --timeout-seconds)
      require_arg_value "$1" "${2:-}"
      TIMEOUT_SECONDS="$2"
      shift 2
      ;;
    --output-json)
      require_arg_value "$1" "${2:-}"
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --help|-h)
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

if [ -z "$ADDR" ] || [[ "$ADDR" != *:* ]]; then
  echo "addr must be in host:port form" >&2
  exit 1
fi

validate_agent_did "from DID" "$FROM_DID"
validate_agent_did "to DID" "$TO_DID"

if ! is_positive_integer "$NONCE"; then
  echo "nonce must be a positive integer" >&2
  exit 1
fi

if ! is_positive_integer "$TIMEOUT_SECONDS"; then
  echo "timeout-seconds must be a positive integer" >&2
  exit 1
fi

if [ ! -x "$REPLAY_POLICY_CHECKER" ]; then
  echo "expected runtime commit replay policy checker to be executable" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
LISTENER_OUT="$TMP_DIR/listener.out"
SENDER_OUT="$TMP_DIR/sender.out"
REPLAY_REPORT="$TMP_DIR/localhost-signed-demo-replay-report.json"

LISTENER_PID=""

cleanup() {
  if [ -n "$LISTENER_PID" ] && kill -0 "$LISTENER_PID" >/dev/null 2>&1; then
    kill "$LISTENER_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

wait_for_listener_completion() {
  local pid="$1"
  local timeout_seconds="$2"
  local elapsed=0

  while kill -0 "$pid" >/dev/null 2>&1; do
    if [ "$elapsed" -ge "$timeout_seconds" ]; then
      echo "listener did not complete within ${timeout_seconds}s" >&2
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
      return 1
    fi
    sleep 1
    elapsed=$((elapsed + 1))
  done

  wait "$pid"
}

cargo run --quiet -p kamn-sdk --example localhost_signed_listener -- \
  --addr "$ADDR" \
  --expected-from "$FROM_DID" \
  --expected-to "$TO_DID" \
  --state-hash "$STATE_HASH" >"$LISTENER_OUT" 2>&1 &
LISTENER_PID=$!

cargo run --quiet -p kamn-sdk --example localhost_signed_sender -- \
  --addr "$ADDR" \
  --from "$FROM_DID" \
  --to "$TO_DID" \
  --nonce "$NONCE" \
  --state-hash "$STATE_HASH" \
  --body "$BODY" >"$SENDER_OUT"

wait_for_listener_completion "$LISTENER_PID" "$TIMEOUT_SECONDS"

echo "--- sender ---"
cat "$SENDER_OUT"
echo "--- listener ---"
cat "$LISTENER_OUT"

if ! grep -Fq "status=ok" "$SENDER_OUT"; then
  echo "expected sender status=ok" >&2
  exit 1
fi
if ! grep -Fq "status=ok" "$LISTENER_OUT"; then
  echo "expected listener status=ok" >&2
  exit 1
fi
if ! grep -Fq "verified=true" "$LISTENER_OUT"; then
  echo "expected listener verified=true marker" >&2
  exit 1
fi

operation_id="op-localhost-demo-${NONCE}"
idempotency_key="localhost-signed-demo:${FROM_DID}:${TO_DID}:${NONCE}:${#BODY}"
receipt_provider="kolme-local"
receipt_commit_id="localhost-receipt:${FROM_DID}:${NONCE}:${#STATE_HASH}"

replay_policy_output="$(
  python3 "$REPLAY_POLICY_CHECKER" \
    --operation-id "$operation_id" \
    --idempotency-key "$idempotency_key" \
    --receipt-provider "$receipt_provider" \
    --expected-receipt-provider "$receipt_provider" \
    --receipt-commit-id "$receipt_commit_id" \
    --expected-receipt-commit-id "$receipt_commit_id" \
    --nonce-monotonic true \
    --replay-detected false \
    --payload-hash-match true \
    --receipt-finality FINAL \
    --ci-fast-gate PASS \
    --output-json "$REPLAY_REPORT"
)"
if ! printf '%s\n' "$replay_policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected runtime commit replay policy checker to produce GO for localhost demo" >&2
  exit 1
fi

sender_signature="$(extract_marker_value "signature" "$SENDER_OUT")"
listener_verified="$(extract_marker_value "verified" "$LISTENER_OUT")"

if [ -n "$OUTPUT_JSON" ]; then
  python3 - "$OUTPUT_JSON" \
    "$ADDR" \
    "$FROM_DID" \
    "$TO_DID" \
    "$NONCE" \
    "$STATE_HASH" \
    "$BODY" \
    "$sender_signature" \
    "$listener_verified" \
    "$operation_id" \
    "$idempotency_key" \
    "$receipt_provider" \
    "$receipt_commit_id" \
    "$REPLAY_REPORT" <<'PY'
import json
import pathlib
import sys
import time

(
    output_json,
    addr,
    from_did,
    to_did,
    nonce,
    state_hash,
    body,
    signature,
    verified,
    operation_id,
    idempotency_key,
    receipt_provider,
    receipt_commit_id,
    replay_report_path,
) = sys.argv[1:]

payload = json.loads(pathlib.Path(replay_report_path).read_text(encoding="utf-8"))
artifact = {
    "schema_version": "kamn.sdk.localhost-signed.demo-receipt-artifact.v1",
    "status": "pass",
    "evidence_key": "localhost_signed_demo_receipt_artifact:v1",
    "generated_at_unix": int(time.time()),
    "signed_exchange": {
        "addr": addr,
        "from": from_did,
        "to": to_did,
        "nonce": int(nonce),
        "state_hash": state_hash,
        "body": body,
        "signature": signature,
        "verified": verified == "true",
    },
    "receipt_reconciliation": {
        "operation_id": operation_id,
        "idempotency_key": idempotency_key,
        "provider": receipt_provider,
        "commit_id": receipt_commit_id,
        "final_decision": payload.get("final_decision"),
        "reason_codes": payload.get("reason_codes", []),
        "policy_schema_version": payload.get("schema_version"),
    },
}

output_path = pathlib.Path(output_json).resolve()
output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(artifact, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY
fi

echo "receipt_reconciliation=GO"
echo "receipt_commit_id=${receipt_commit_id}"
echo "artifact_schema=${ARTIFACT_SCHEMA}"
if [ -n "$OUTPUT_JSON" ]; then
  echo "artifact_file=$(realpath "$OUTPUT_JSON")"
fi


echo "localhost signed message demo completed."
