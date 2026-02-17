#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

output_json=""
max_seconds=180

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
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

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"

pushd "$ROOT_DIR" >/dev/null
cargo build --quiet -p kamn-node
NODE_BIN="$ROOT_DIR/target/debug/kamn-node"
popd >/dev/null

if [ ! -x "$NODE_BIN" ]; then
  echo "expected built kamn-node binary to be executable" >&2
  exit 1
fi

api_port="$(python3 - <<'PY'
import socket
sock = socket.socket()
sock.bind(("127.0.0.1", 0))
print(sock.getsockname()[1])
sock.close()
PY
)"
api_addr="127.0.0.1:${api_port}"

api_stdout="$TMP_DIR/service-api-auth.out"
"$NODE_BIN" \
  --role processor \
  --chain-id kamn-devnet \
  --chain-version v0.1.0 \
  --runtime-mode api \
  --api-bind "$api_addr" \
  --api-max-requests 6 \
  --api-idle-timeout-ms 5000 \
  --output json >"$api_stdout" 2>&1 &
node_pid=$!

auth_sender_did="kamn:did:agent:request-auth-validator"
auth_state_hash="service-api:kamn-devnet:v0.1.0"
auth_body='{"message":"auth-drill"}'

build_signature() {
  local nonce="$1"
  local payload="$2"
  printf 'sig:ed25519:baseline-v1:%s:%s:%s:%s' \
    "$auth_sender_did" \
    "$nonce" \
    "$auth_state_hash" \
    "${#payload}"
}

wait_for_ready=0
for _ in $(seq 1 120); do
  if curl -fsS "http://${api_addr}/healthz" >/dev/null 2>&1; then
    wait_for_ready=1
    break
  fi
  if ! kill -0 "$node_pid" 2>/dev/null; then
    break
  fi
  sleep 0.05
done

if [ "$wait_for_ready" -ne 1 ]; then
  cat "$api_stdout" >&2
  echo "expected service api auth endpoint to become ready" >&2
  kill -KILL "$node_pid" 2>/dev/null || true
  wait "$node_pid" 2>/dev/null || true
  exit 1
fi

unauthorized_file="$TMP_DIR/unauthorized.json"
authorized_file="$TMP_DIR/authorized.json"
replay_file="$TMP_DIR/replay.json"
health_file="$TMP_DIR/health.json"
metrics_file="$TMP_DIR/metrics.txt"

unauthorized_status="$(curl -sS -o "$unauthorized_file" -w '%{http_code}' \
  -X POST "http://${api_addr}/v1/messages/send" \
  -H 'content-type: application/json' \
  --data "$auth_body")"
if [ "$unauthorized_status" != "401" ]; then
  cat "$unauthorized_file" >&2
  echo "expected unauthorized request to return 401" >&2
  exit 1
fi
if ! grep -q '"error":"unauthorized"' "$unauthorized_file"; then
  cat "$unauthorized_file" >&2
  echo "expected unauthorized response marker" >&2
  exit 1
fi
if ! grep -q 'x-kamn-sender-did' "$unauthorized_file"; then
  cat "$unauthorized_file" >&2
  echo "expected missing sender header reason marker" >&2
  exit 1
fi

nonce=1
signature="$(build_signature "$nonce" "$auth_body")"
authorized_status="$(curl -sS -o "$authorized_file" -w '%{http_code}' \
  -X POST "http://${api_addr}/v1/messages/send" \
  -H 'content-type: application/json' \
  -H "X-KAMN-Sender-DID: ${auth_sender_did}" \
  -H "X-KAMN-Request-Nonce: ${nonce}" \
  -H "X-KAMN-Request-Signature: ${signature}" \
  --data "$auth_body")"
if [ "$authorized_status" != "202" ]; then
  cat "$authorized_file" >&2
  echo "expected authorized request to return 202" >&2
  exit 1
fi
if ! grep -q '"status":"created"' "$authorized_file"; then
  cat "$authorized_file" >&2
  echo "expected authorized response creation marker" >&2
  exit 1
fi

replay_status="$(curl -sS -o "$replay_file" -w '%{http_code}' \
  -X POST "http://${api_addr}/v1/messages/send" \
  -H 'content-type: application/json' \
  -H "X-KAMN-Sender-DID: ${auth_sender_did}" \
  -H "X-KAMN-Request-Nonce: ${nonce}" \
  -H "X-KAMN-Request-Signature: ${signature}" \
  --data "$auth_body")"
if [ "$replay_status" != "409" ]; then
  cat "$replay_file" >&2
  echo "expected replayed request to return 409" >&2
  exit 1
fi
if ! grep -q '"error":"replay"' "$replay_file"; then
  cat "$replay_file" >&2
  echo "expected replay response marker" >&2
  exit 1
fi

health_status="$(curl -sS -o "$health_file" -w '%{http_code}' "http://${api_addr}/healthz")"
if [ "$health_status" != "200" ]; then
  cat "$health_file" >&2
  echo "expected health probe to return 200" >&2
  exit 1
fi
if ! grep -q '"status":"ok"' "$health_file"; then
  cat "$health_file" >&2
  echo "expected health payload status marker" >&2
  exit 1
fi

metrics_status="$(curl -sS -o "$metrics_file" -w '%{http_code}' "http://${api_addr}/metrics")"
if [ "$metrics_status" != "200" ]; then
  cat "$metrics_file" >&2
  echo "expected metrics probe to return 200" >&2
  exit 1
fi
if ! grep -q 'kamn_service_api_health' "$metrics_file"; then
  cat "$metrics_file" >&2
  echo "expected metrics payload marker" >&2
  exit 1
fi

set +e
wait "$node_pid"
node_exit_code=$?
set -e
if [ "$node_exit_code" -ne 0 ]; then
  cat "$api_stdout" >&2
  echo "expected service api auth process to exit cleanly after request budget" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "service api request-auth live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/service-api-request-auth-live-validation-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.runtime.service-api-request-auth-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "unauthorized_guard_status": "verified",
  "replay_guard_status": "verified",
  "probe_status": "verified",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "unauthorized_guard_status=verified"
echo "replay_guard_status=verified"
echo "probe_status=verified"
