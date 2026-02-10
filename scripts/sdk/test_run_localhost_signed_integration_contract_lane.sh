#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/sdk/run_localhost_signed_integration_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/sdk/localhost_signed_integration_contract_lane_contract.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected localhost signed integration contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$SHARED_CONTRACT" ]; then
  echo "expected localhost signed integration shared contract lane module to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/localhost-signed-integration-contract.json"
lane_output="$(
  bash "$LANE_SCRIPT" --output-json "$report_file"
)"

required_markers=(
  "localhost_signed_integration_success=pass"
  "localhost_signed_integration_signature_mismatch=pass"
  "localhost_signed_integration_timeout=pass"
  "localhost_signed_integration_replay_nonce=pass"
  "localhost_signed_integration_admission_guards=pass"
  "localhost_signed_integration_policy=ok"
  "localhost signed integration contract lane tests passed."
)

for marker in "${required_markers[@]}"; do
  if ! printf '%s\n' "$lane_output" | grep -Fq -- "$marker"; then
    echo "expected localhost signed integration contract lane output marker '$marker'" >&2
    exit 1
  fi
done

python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["schema_version"] == "kamn.sdk.localhost-signed.integration-contract.v1"
assert report["status"] == "pass"
assert report["success_scenario_status"] == "pass"
assert report["signature_mismatch_scenario_status"] == "pass"
assert report["timeout_scenario_status"] == "pass"
assert report["replay_nonce_scenario_status"] == "pass"
assert report["admission_guards_scenario_status"] == "pass"
assert report["contract_key"] == "localhost_signed_integration_contract:v1"
assert report["success_evidence_key"] == "localhost_signed_integration:success:v1"
assert (
    report["signature_mismatch_evidence_key"]
    == "localhost_signed_integration:signature-mismatch:v1"
)
assert report["timeout_evidence_key"] == "localhost_signed_integration:timeout:v1"
assert report["replay_nonce_evidence_key"] == "localhost_signed_integration:replay-nonce:v1"
assert (
    report["admission_guards_evidence_key"]
    == "localhost_signed_integration:admission-guards:v1"
)
assert report["success_reason_key"] == "localhost_signed_integration_reason:none:v1"
assert (
    report["signature_mismatch_reason_key"]
    == "localhost_signed_integration_reason:signature_mismatch_detected:v1"
)
assert (
    report["timeout_reason_key"]
    == "localhost_signed_integration_reason:listener_timeout_detected:v1"
)
assert (
    report["replay_nonce_reason_key"]
    == "localhost_signed_integration_reason:replay_nonce_detected:v1"
)
assert (
    report["admission_guards_reason_key"]
    == "localhost_signed_integration_reason:session_admission_guards_detected:v1"
)
# Regression: #878
assert report["signature_mismatch_reason_code"] == "signature_mismatch_detected"
assert report["timeout_reason_code"] == "listener_timeout_detected"
assert report["replay_nonce_reason_code"] == "replay_nonce_detected"
assert (
    report["admission_guards_reason_code"] == "session_admission_guards_detected"
)
assert report["replay_guard_status"] == "pass"
assert report["replay_rejected_nonce"] == 7
assert report["admission_guard_status"] == "pass"
assert report["admission_reason_codes"] == [
    "stale_session_detected",
    "unauthorized_sender_detected",
    "malformed_payload_detected",
]
PY

if ! grep -Fq "localhost_signed_integration_contract_lane_contract.py" "$LANE_SCRIPT"; then
  echo "expected localhost signed integration contract lane wrapper to dispatch shared contract module" >&2
  exit 1
fi

if ! grep -Fq "check_localhost_signed_integration_evidence_policy.sh" "$SHARED_CONTRACT"; then
  echo "expected localhost signed integration shared contract lane module to enforce evidence policy checker" >&2
  exit 1
fi

echo "localhost signed integration contract lane script tests passed."
