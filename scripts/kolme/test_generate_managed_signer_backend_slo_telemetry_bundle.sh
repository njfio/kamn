#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/kolme/generate_managed_signer_backend_slo_telemetry_bundle.sh"
TMP_DIR="$(mktemp -d)"
TMP_ERR="$TMP_DIR/error.log"
TMP_GO_BUNDLE="$TMP_DIR/managed-signer-slo-go.json"
TMP_NO_GO_BUNDLE="$TMP_DIR/managed-signer-slo-no-go.json"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected managed-signer backend SLO telemetry bundle generator to be executable" >&2
  exit 1
fi

go_output="$(
  bash "$GENERATOR" \
    --output-file "$TMP_GO_BUNDLE" \
    --window-start-utc "2026-02-13T00:00:00Z" \
    --window-end-utc "2026-02-13T00:15:00Z" \
    --backend-name "kolme-managed-signer-primary" \
    --signer-profile "ops-primary" \
    --signer-key-source "managed-external" \
    --sample-count 100 \
    --timeout-events 0 \
    --unavailable-events 0 \
    --error-events 1 \
    --max-timeout-rate-bps 100 \
    --max-unavailable-rate-bps 100 \
    --max-error-rate-bps 200 \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$go_output" | grep -q "^status=generated$"; then
  echo "expected managed-signer SLO generator status=generated for GO scenario" >&2
  exit 1
fi

if ! printf '%s\n' "$go_output" | grep -q "^final_decision=GO$"; then
  echo "expected managed-signer SLO generator GO decision for healthy scenario" >&2
  exit 1
fi

python3 - "$TMP_GO_BUNDLE" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.kolme.managed-signer-backend-slo-telemetry.v1":
    raise SystemExit("unexpected managed-signer SLO telemetry schema version")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected GO decision in healthy managed-signer SLO telemetry bundle")
if payload.get("threshold_breaches") != []:
    raise SystemExit("expected no threshold breaches in healthy managed-signer SLO telemetry bundle")
contracts = payload.get("contracts")
if not isinstance(contracts, dict):
    raise SystemExit("expected contracts object in managed-signer SLO telemetry bundle")
if contracts.get("required_signer_key_source") != "managed-external":
    raise SystemExit("expected required managed-signer key source contract marker")
if contracts.get("threshold_source") != "operator-slo-policy":
    raise SystemExit("expected threshold source contract marker")
PY

no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$TMP_NO_GO_BUNDLE" \
    --window-start-utc "2026-02-13T00:15:00Z" \
    --window-end-utc "2026-02-13T00:30:00Z" \
    --backend-name "kolme-managed-signer-primary" \
    --signer-profile "ops-primary" \
    --signer-key-source "managed-external" \
    --sample-count 100 \
    --timeout-events 8 \
    --unavailable-events 7 \
    --error-events 9 \
    --max-timeout-rate-bps 500 \
    --max-unavailable-rate-bps 500 \
    --max-error-rate-bps 500 \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$no_go_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected managed-signer SLO generator NO-GO decision for threshold breach scenario" >&2
  exit 1
fi

python3 - "$TMP_NO_GO_BUNDLE" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected NO-GO decision in failing managed-signer SLO telemetry bundle")
breaches = payload.get("threshold_breaches")
if not isinstance(breaches, list):
    raise SystemExit("expected threshold_breaches list in failing managed-signer SLO telemetry bundle")
expected_breaches = {
    "managed_signer_backend_timeout_rate_threshold_exceeded",
    "managed_signer_backend_unavailable_rate_threshold_exceeded",
    "managed_signer_backend_error_rate_threshold_exceeded",
}
if set(breaches) != expected_breaches:
    raise SystemExit("expected all managed-signer backend threshold breach reason codes in failing bundle")
PY

set +e
bash "$GENERATOR" \
  --output-file "$TMP_DIR/malformed.json" \
  --window-start-utc "2026-02-13T00:30:00Z" \
  --window-end-utc "2026-02-13T00:45:00Z" \
  --backend-name "kolme-managed-signer-primary" \
  --signer-profile "ops-primary" \
  --signer-key-source "managed-external" \
  --sample-count 10 \
  --timeout-events 0 \
  --unavailable-events 11 \
  --error-events 0 \
  --max-timeout-rate-bps 100 \
  --max-unavailable-rate-bps 100 \
  --max-error-rate-bps 100 \
  --ci-fast-gate PASS >"$TMP_ERR" 2>&1
malformed_exit_code=$?
set -e

if [ "$malformed_exit_code" -eq 0 ]; then
  echo "expected managed-signer SLO generator to fail on malformed event counts" >&2
  exit 1
fi

if ! grep -q "must be <= sample-count" "$TMP_ERR"; then
  echo "expected explicit sample-count bound failure for malformed managed-signer SLO input" >&2
  exit 1
fi

echo "managed-signer backend SLO telemetry bundle tests passed."
