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

api_stdout="$TMP_DIR/service-api-observability.out"
"$NODE_BIN" \
  --role processor \
  --chain-id kamn-devnet \
  --chain-version v0.1.0 \
  --runtime-mode api \
  --api-bind "$api_addr" \
  --api-max-requests 3 \
  --api-idle-timeout-ms 5000 \
  --output json >"$api_stdout" 2>&1 &
node_pid=$!

ready=0
for _ in $(seq 1 120); do
  if grep -q 'node.runtime.service_api.endpoint.start' "$api_stdout"; then
    ready=1
    break
  fi
  if ! kill -0 "$node_pid" 2>/dev/null; then
    break
  fi
  sleep 0.05
done

if [ "$ready" -ne 1 ]; then
  cat "$api_stdout" >&2
  echo "expected service api observability endpoint to start" >&2
  kill -KILL "$node_pid" 2>/dev/null || true
  wait "$node_pid" 2>/dev/null || true
  exit 1
fi

health_file="$TMP_DIR/healthz.json"
metrics_file="$TMP_DIR/metrics.txt"
post_metrics_file="$TMP_DIR/post-metrics.txt"

health_status="$(curl -sS -o "$health_file" -w '%{http_code}' "http://${api_addr}/healthz")"
if [ "$health_status" != "200" ]; then
  cat "$health_file" >&2
  echo "expected health endpoint to return 200" >&2
  exit 1
fi
if ! grep -q '"status":"ok"' "$health_file"; then
  cat "$health_file" >&2
  echo "expected health status marker" >&2
  exit 1
fi
if ! grep -q '"observability_source":"unknown"' "$health_file"; then
  cat "$health_file" >&2
  echo "expected health observability source marker" >&2
  exit 1
fi
if ! grep -q '"observability_health":"unknown"' "$health_file"; then
  cat "$health_file" >&2
  echo "expected health observability marker" >&2
  exit 1
fi

metrics_status="$(curl -sS -o "$metrics_file" -w '%{http_code}' "http://${api_addr}/metrics")"
if [ "$metrics_status" != "200" ]; then
  cat "$metrics_file" >&2
  echo "expected metrics endpoint to return 200" >&2
  exit 1
fi
if ! grep -q 'kamn_service_api_observability_source{source="unknown"} 1' "$metrics_file"; then
  cat "$metrics_file" >&2
  echo "expected observability source metrics marker" >&2
  exit 1
fi
if ! grep -q 'kamn_service_api_observability_health{health="unknown"} 0' "$metrics_file"; then
  cat "$metrics_file" >&2
  echo "expected observability health metrics marker" >&2
  exit 1
fi
if ! grep -q 'kamn_service_api_observability_latency_p50_ms 0' "$metrics_file"; then
  cat "$metrics_file" >&2
  echo "expected observability latency metrics marker" >&2
  exit 1
fi

sender_did="kamn:did:agent:service-api-observability-validator"
nonce=1
state_hash="service-api:kamn-devnet:v0.1.0"
signature="sig:ed25519:baseline-v1:${sender_did}:${nonce}:${state_hash}:0"
post_metrics_status="$(curl -sS -o "$post_metrics_file" -w '%{http_code}' \
  -X POST "http://${api_addr}/metrics" \
  -H "X-KAMN-Sender-DID: ${sender_did}" \
  -H "X-KAMN-Request-Nonce: ${nonce}" \
  -H "X-KAMN-Request-Signature: ${signature}" \
  --data '')"
if [ "$post_metrics_status" != "405" ]; then
  cat "$post_metrics_file" >&2
  echo "expected invalid method on /metrics to fail closed with 405" >&2
  exit 1
fi
if ! grep -q 'method not allowed' "$post_metrics_file"; then
  cat "$post_metrics_file" >&2
  echo "expected fail-closed method marker for /metrics" >&2
  exit 1
fi

set +e
wait "$node_pid"
node_exit_code=$?
set -e
if [ "$node_exit_code" -ne 0 ]; then
  cat "$api_stdout" >&2
  echo "expected service api observability process to exit cleanly after request budget" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "service api observability live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/service-api-observability-live-validation-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.runtime.service-api-observability-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "metrics_contract_status": "verified",
  "health_contract_status": "verified",
  "fail_closed_status": "verified",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "metrics_contract_status=verified"
echo "health_contract_status=verified"
echo "fail_closed_status=verified"
