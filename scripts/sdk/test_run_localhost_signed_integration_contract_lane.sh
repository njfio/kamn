#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/sdk/run_localhost_signed_integration_contract_lane.sh"
SHARED_CONTRACT="$ROOT_DIR/scripts/sdk/localhost_signed_integration_contract_lane_contract.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/runtime/localhost_signed_integration_cases.json"
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

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected localhost signed integration fixture corpus to exist" >&2
  exit 1
fi

report_file="$TMP_DIR/localhost-signed-integration-contract.json"
lane_output="$(
  bash "$LANE_SCRIPT" --output-json "$report_file"
)"

required_markers=(
  "localhost_signed_integration_success=pass"
  "localhost_signed_integration_signature_mismatch=pass"
  "localhost_signed_integration_malformed_signature=pass"
  "localhost_signed_integration_timeout=pass"
  "localhost_signed_integration_session_expired=pass"
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
assert report["final_decision"] == "GO"
assert report["success_scenario_status"] == "pass"
assert report["signature_mismatch_scenario_status"] == "pass"
assert report["malformed_signature_scenario_status"] == "pass"
assert report["timeout_scenario_status"] == "pass"
assert report["session_expired_scenario_status"] == "pass"
assert report["replay_nonce_scenario_status"] == "pass"
assert report["admission_guards_scenario_status"] == "pass"
assert (
    report["scenario_fixture_schema_version"]
    == "kamn.sdk.localhost-signed.integration-fixtures.v1"
)
assert report["scenario_fixture_ids"] == [
    "success-v1",
    "signature-mismatch-v1",
    "timeout-v1",
]
assert report["contract_key"] == "localhost_signed_integration_contract:v1"
assert report["success_evidence_key"] == "localhost_signed_integration:success:v1"
assert (
    report["signature_mismatch_evidence_key"]
    == "localhost_signed_integration:signature-mismatch:v1"
)
assert (
    report["malformed_signature_evidence_key"]
    == "localhost_signed_integration:malformed-signature:v1"
)
assert report["timeout_evidence_key"] == "localhost_signed_integration:timeout:v1"
assert (
    report["session_expired_evidence_key"]
    == "localhost_signed_integration:session-expired:v1"
)
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
    report["malformed_signature_reason_key"]
    == "localhost_signed_integration_reason:malformed_signature_detected:v1"
)
assert (
    report["timeout_reason_key"]
    == "localhost_signed_integration_reason:listener_timeout_detected:v1"
)
assert (
    report["session_expired_reason_key"]
    == "localhost_signed_integration_reason:session_expired_detected:v1"
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
assert report["malformed_signature_reason_code"] == "malformed_signature_detected"
assert report["timeout_reason_code"] == "listener_timeout_detected"
assert report["session_expired_reason_code"] == "session_expired_detected"
assert report["replay_nonce_reason_code"] == "replay_nonce_detected"
assert (
    report["admission_guards_reason_code"] == "session_admission_guards_detected"
)
assert report["expiry_guard_status"] == "pass"
assert report["replay_guard_status"] == "pass"
assert report["replay_rejected_nonce"] == 7
assert report["admission_guard_status"] == "pass"
assert report["admission_reason_codes"] == [
    "stale_session_detected",
    "unauthorized_sender_detected",
    "malformed_payload_detected",
]
PY

python3 - "$FIXTURE_FILE" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert payload["schema_version"] == "kamn.sdk.localhost-signed.integration-fixtures.v1"
assert payload["scenario_ids"] == [
    "success-v1",
    "signature-mismatch-v1",
    "timeout-v1",
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

if ! grep -Fq "fixtures/runtime/localhost_signed_integration_cases.json" "$SHARED_CONTRACT"; then
  echo "expected localhost signed integration shared contract lane module to enforce fixture corpus contract" >&2
  exit 1
fi

echo "localhost signed integration contract lane script tests passed."
