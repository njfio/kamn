#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/sdk/run_localhost_signed_integration_harness.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected localhost signed integration harness runner to be executable" >&2
  exit 1
fi

success_report="$TMP_DIR/success.json"
success_output="$(
  bash "$RUNNER" \
    --scenario success \
    --output-json "$success_report"
)"
if ! printf '%s\n' "$success_output" | grep -Fq "status=pass; scenario=success; reason_code=none;"; then
  echo "expected success scenario summary from localhost signed integration harness" >&2
  exit 1
fi

signature_report="$TMP_DIR/signature-mismatch.json"
signature_output="$(
  bash "$RUNNER" \
    --scenario signature-mismatch \
    --output-json "$signature_report"
)"
if ! printf '%s\n' "$signature_output" | grep -Fq "status=pass; scenario=signature-mismatch; reason_code=signature_mismatch_detected;"; then
  echo "expected signature mismatch scenario summary from localhost signed integration harness" >&2
  exit 1
fi

timeout_report="$TMP_DIR/timeout.json"
timeout_output="$(
  bash "$RUNNER" \
    --scenario timeout \
    --timeout-seconds 1 \
    --output-json "$timeout_report"
)"
if ! printf '%s\n' "$timeout_output" | grep -Fq "status=pass; scenario=timeout; reason_code=listener_timeout_detected;"; then
  echo "expected timeout scenario summary from localhost signed integration harness" >&2
  exit 1
fi

python3 - "$success_report" "$signature_report" "$timeout_report" <<'PY'
import json
import pathlib
import sys

success_report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
signature_report = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
timeout_report = json.loads(pathlib.Path(sys.argv[3]).read_text(encoding="utf-8"))

assert success_report["schema_version"] == "kamn.sdk.localhost-signed.integration-harness.v1"
assert success_report["status"] == "pass"
assert success_report["scenario"] == "success"
assert success_report["reason_code"] == "none"
assert success_report["elapsed_seconds"] >= 0

assert signature_report["status"] == "pass"
assert signature_report["scenario"] == "signature-mismatch"
# Regression: #876
assert signature_report["reason_code"] == "signature_mismatch_detected"

assert timeout_report["status"] == "pass"
assert timeout_report["scenario"] == "timeout"
assert timeout_report["reason_code"] == "listener_timeout_detected"
PY

echo "localhost signed integration harness tests passed."
