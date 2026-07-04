#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEMO_SCRIPT="$ROOT_DIR/scripts/sdk/run_localhost_signed_demo.sh"

if [ ! -x "$DEMO_SCRIPT" ]; then
  echo "expected localhost signed demo runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
TMP_HELP="$(mktemp)"
TMP_ERR="$(mktemp)"
TMP_ARTIFACT="$(mktemp)"
trap 'rm -f "$TMP_OUT" "$TMP_HELP" "$TMP_ERR" "$TMP_ARTIFACT"' EXIT

bash "$DEMO_SCRIPT" --help >"$TMP_HELP"

if ! grep -Fq -- "Usage: run_localhost_signed_demo.sh" "$TMP_HELP"; then
  echo "expected localhost signed demo help usage banner" >&2
  exit 1
fi

if ! grep -Fq -- "--timeout-seconds" "$TMP_HELP"; then
  echo "expected localhost signed demo help to document --timeout-seconds" >&2
  exit 1
fi

if ! grep -Fq -- "--output-json" "$TMP_HELP"; then
  echo "expected localhost signed demo help to document --output-json" >&2
  exit 1
fi

if grep -Fq 'KAMN_LOCALHOST_SIGNED_DEMO_ADDR:-127.0.0.1:17879' "$DEMO_SCRIPT"; then
  echo "expected localhost signed demo to avoid fixed default port 17879" >&2
  exit 1
fi

if ! grep -Fq 'KAMN_LOCALHOST_SIGNED_DEMO_ADDR:-127.0.0.1:0' "$DEMO_SCRIPT"; then
  echo "expected localhost signed demo default to ask the listener for an ephemeral port" >&2
  exit 1
fi

if ! grep -Fq 'extract_marker_value "addr" "$LISTENER_OUT"' "$DEMO_SCRIPT"; then
  echo "expected localhost signed demo to reuse the listener-reported bound address" >&2
  exit 1
fi

set +e
bash "$DEMO_SCRIPT" --timeout-seconds 0 >"$TMP_ERR" 2>&1
error_code=$?
set -e

if [ "$error_code" -eq 0 ]; then
  echo "expected localhost signed demo script to reject invalid timeout argument" >&2
  exit 1
fi

# Regression: #875
if ! grep -Fq -- "timeout-seconds must be a positive integer" "$TMP_ERR"; then
  echo "expected explicit timeout validation failure message" >&2
  exit 1
fi

bash "$DEMO_SCRIPT" --output-json "$TMP_ARTIFACT" >"$TMP_OUT"

required_markers=(
  "--- sender ---"
  "--- listener ---"
  "status=ok"
  "verified=true"
  "signature=sig:deterministic-v1:baseline-v1:"
  "receipt_reconciliation=GO"
  "artifact_schema=kamn.sdk.localhost-signed.demo-receipt-artifact.v1"
  "localhost signed message demo completed."
)

for marker in "${required_markers[@]}"; do
  if ! grep -Fq -- "$marker" "$TMP_OUT"; then
    echo "expected localhost signed demo output marker '$marker'" >&2
    exit 1
  fi
done

python3 - "$TMP_ARTIFACT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.sdk.localhost-signed.demo-receipt-artifact.v1":
    raise SystemExit("unexpected localhost signed demo artifact schema")
if payload.get("status") != "pass":
    raise SystemExit("expected localhost signed demo artifact status=pass")
exchange = payload.get("signed_exchange", {})
if exchange.get("from") != "kamn:did:agent:sender-1":
    raise SystemExit("expected signed_exchange.from to match sender DID")
if exchange.get("to") != "kamn:did:agent:listener-1":
    raise SystemExit("expected signed_exchange.to to match listener DID")
if exchange.get("verified") is not True:
    raise SystemExit("expected signed_exchange.verified=true")
addr = exchange.get("addr")
if not isinstance(addr, str) or not addr.startswith("127.0.0.1:"):
    raise SystemExit("expected signed_exchange.addr to use loopback")
if addr.endswith(":0") or addr.endswith(":17879"):
    raise SystemExit("expected signed_exchange.addr to be a dynamic bound port")
receipt = payload.get("receipt_reconciliation", {})
if receipt.get("final_decision") != "GO":
    raise SystemExit("expected receipt_reconciliation.final_decision=GO")
if receipt.get("reason_codes") != []:
    raise SystemExit("expected receipt_reconciliation.reason_codes to be empty list")
if not receipt.get("commit_id"):
    raise SystemExit("expected non-empty receipt_reconciliation.commit_id")
PY

echo "localhost signed demo script tests passed."
