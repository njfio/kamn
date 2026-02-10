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
if ! printf '%s\n' "$success_output" | grep -Fq "status=pass; scenario=success;"; then
  echo "expected success scenario status summary from localhost signed integration harness" >&2
  exit 1
fi
if ! printf '%s\n' "$success_output" | grep -Fq "reason_code=none;"; then
  echo "expected success scenario reason code from localhost signed integration harness" >&2
  exit 1
fi
if ! printf '%s\n' "$success_output" | grep -Fq "evidence_key=localhost_signed_integration:success:v1;"; then
  echo "expected success scenario evidence key from localhost signed integration harness" >&2
  exit 1
fi

signature_report="$TMP_DIR/signature-mismatch.json"
signature_output="$(
  bash "$RUNNER" \
    --scenario signature-mismatch \
    --output-json "$signature_report"
)"
if ! printf '%s\n' "$signature_output" | grep -Fq "status=pass; scenario=signature-mismatch;"; then
  echo "expected signature mismatch scenario status summary from localhost signed integration harness" >&2
  exit 1
fi
if ! printf '%s\n' "$signature_output" | grep -Fq "reason_code=signature_mismatch_detected;"; then
  echo "expected signature mismatch scenario reason code from localhost signed integration harness" >&2
  exit 1
fi
if ! printf '%s\n' "$signature_output" | grep -Fq "evidence_key=localhost_signed_integration:signature-mismatch:v1;"; then
  echo "expected signature mismatch scenario evidence key from localhost signed integration harness" >&2
  exit 1
fi

timeout_report="$TMP_DIR/timeout.json"
timeout_output="$(
  bash "$RUNNER" \
    --scenario timeout \
    --timeout-seconds 1 \
    --output-json "$timeout_report"
)"
if ! printf '%s\n' "$timeout_output" | grep -Fq "status=pass; scenario=timeout;"; then
  echo "expected timeout scenario status summary from localhost signed integration harness" >&2
  exit 1
fi
if ! printf '%s\n' "$timeout_output" | grep -Fq "reason_code=listener_timeout_detected;"; then
  echo "expected timeout scenario reason code from localhost signed integration harness" >&2
  exit 1
fi
if ! printf '%s\n' "$timeout_output" | grep -Fq "evidence_key=localhost_signed_integration:timeout:v1;"; then
  echo "expected timeout scenario evidence key from localhost signed integration harness" >&2
  exit 1
fi

replay_report="$TMP_DIR/replay-nonce.json"
replay_output="$(
  bash "$RUNNER" \
    --scenario replay-nonce \
    --output-json "$replay_report"
)"
if ! printf '%s\n' "$replay_output" | grep -Fq "status=pass; scenario=replay-nonce;"; then
  echo "expected replay nonce scenario status summary from localhost signed integration harness" >&2
  exit 1
fi
if ! printf '%s\n' "$replay_output" | grep -Fq "reason_code=replay_nonce_detected;"; then
  echo "expected replay nonce scenario reason code from localhost signed integration harness" >&2
  exit 1
fi
if ! printf '%s\n' "$replay_output" | grep -Fq "evidence_key=localhost_signed_integration:replay-nonce:v1;"; then
  echo "expected replay nonce scenario evidence key from localhost signed integration harness" >&2
  exit 1
fi

admission_report="$TMP_DIR/admission-guards.json"
admission_output="$(
  bash "$RUNNER" \
    --scenario admission-guards \
    --output-json "$admission_report"
)"
if ! printf '%s\n' "$admission_output" | grep -Fq "status=pass; scenario=admission-guards;"; then
  echo "expected admission guards scenario status summary from localhost signed integration harness" >&2
  exit 1
fi
if ! printf '%s\n' "$admission_output" | grep -Fq "reason_code=session_admission_guards_detected;"; then
  echo "expected admission guards scenario reason code from localhost signed integration harness" >&2
  exit 1
fi
if ! printf '%s\n' "$admission_output" | grep -Fq "evidence_key=localhost_signed_integration:admission-guards:v1;"; then
  echo "expected admission guards scenario evidence key from localhost signed integration harness" >&2
  exit 1
fi

python3 - \
  "$success_report" \
  "$signature_report" \
  "$timeout_report" \
  "$replay_report" \
  "$admission_report" <<'PY'
import json
import pathlib
import sys

success_report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
signature_report = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
timeout_report = json.loads(pathlib.Path(sys.argv[3]).read_text(encoding="utf-8"))
replay_report = json.loads(pathlib.Path(sys.argv[4]).read_text(encoding="utf-8"))
admission_report = json.loads(pathlib.Path(sys.argv[5]).read_text(encoding="utf-8"))

assert success_report["schema_version"] == "kamn.sdk.localhost-signed.integration-harness.v1"
assert success_report["status"] == "pass"
assert success_report["scenario"] == "success"
assert success_report["reason_code"] == "none"
assert success_report["evidence_key"] == "localhost_signed_integration:success:v1"
assert success_report["reason_key"] == "localhost_signed_integration_reason:none:v1"
assert success_report["elapsed_seconds"] >= 0

assert signature_report["status"] == "pass"
assert signature_report["scenario"] == "signature-mismatch"
# Regression: #876
assert signature_report["reason_code"] == "signature_mismatch_detected"
assert (
    signature_report["evidence_key"]
    == "localhost_signed_integration:signature-mismatch:v1"
)
assert (
    signature_report["reason_key"]
    == "localhost_signed_integration_reason:signature_mismatch_detected:v1"
)

assert timeout_report["status"] == "pass"
assert timeout_report["scenario"] == "timeout"
assert timeout_report["reason_code"] == "listener_timeout_detected"
assert timeout_report["evidence_key"] == "localhost_signed_integration:timeout:v1"
assert (
    timeout_report["reason_key"]
    == "localhost_signed_integration_reason:listener_timeout_detected:v1"
)

assert replay_report["status"] == "pass"
assert replay_report["scenario"] == "replay-nonce"
assert replay_report["reason_code"] == "replay_nonce_detected"
assert replay_report["evidence_key"] == "localhost_signed_integration:replay-nonce:v1"
assert replay_report["replay_guard_status"] == "pass"
assert replay_report["replay_rejected_nonce"] == 7
assert (
    replay_report["reason_key"]
    == "localhost_signed_integration_reason:replay_nonce_detected:v1"
)

assert admission_report["status"] == "pass"
assert admission_report["scenario"] == "admission-guards"
assert admission_report["reason_code"] == "session_admission_guards_detected"
assert (
    admission_report["evidence_key"]
    == "localhost_signed_integration:admission-guards:v1"
)
assert admission_report["admission_guard_status"] == "pass"
assert admission_report["admission_reason_codes"] == [
    "stale_session_detected",
    "unauthorized_sender_detected",
    "malformed_payload_detected",
]
assert (
    admission_report["reason_key"]
    == "localhost_signed_integration_reason:session_admission_guards_detected:v1"
)
PY

echo "localhost signed integration harness tests passed."
