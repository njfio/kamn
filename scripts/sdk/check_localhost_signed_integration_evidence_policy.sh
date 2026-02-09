#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/sdk/check_localhost_signed_integration_evidence_policy.sh \
    --report-file <path>
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

report_file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --report-file)
      report_file="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if [[ -z "$report_file" ]]; then
  usage
  fail "--report-file is required"
fi

if [[ ! -f "$report_file" ]]; then
  fail "report file not found: $report_file"
fi

output="$(
  python3 - "$report_file" <<'PY'
import json
import pathlib
import sys


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    sys.exit(1)


report_path = pathlib.Path(sys.argv[1])
try:
    payload = json.loads(report_path.read_text(encoding="utf-8"))
except json.JSONDecodeError as exc:
    fail(f"report file is not valid JSON: {exc}")

required_fields = (
    "schema_version",
    "status",
    "contract_key",
    "success_scenario_status",
    "signature_mismatch_scenario_status",
    "timeout_scenario_status",
    "success_evidence_key",
    "signature_mismatch_evidence_key",
    "timeout_evidence_key",
    "signature_mismatch_reason_code",
    "timeout_reason_code",
    "success_reason_key",
    "signature_mismatch_reason_key",
    "timeout_reason_key",
    "success_elapsed_seconds",
    "signature_mismatch_elapsed_seconds",
    "timeout_elapsed_seconds",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing report field: {field}")

if payload["schema_version"] != "kamn.sdk.localhost-signed.integration-contract.v1":
    fail("unexpected schema_version for localhost signed integration contract report")

status = payload["status"]
if status not in {"pass", "fail"}:
    fail("status must be pass or fail")

if payload["contract_key"] != "localhost_signed_integration_contract:v1":
    fail("contract_key must be localhost_signed_integration_contract:v1")

for field in (
    "success_scenario_status",
    "signature_mismatch_scenario_status",
    "timeout_scenario_status",
):
    if payload[field] not in {"pass", "fail"}:
        fail(f"{field} must be pass or fail")

if payload["signature_mismatch_reason_code"] != "signature_mismatch_detected":
    fail(
        "signature_mismatch_reason_code must be signature_mismatch_detected"
    )

if payload["timeout_reason_code"] != "listener_timeout_detected":
    fail("timeout_reason_code must be listener_timeout_detected")

if payload["success_evidence_key"] != "localhost_signed_integration:success:v1":
    fail("success_evidence_key must be localhost_signed_integration:success:v1")

if (
    payload["signature_mismatch_evidence_key"]
    != "localhost_signed_integration:signature-mismatch:v1"
):
    fail(
        "signature_mismatch_evidence_key must be "
        "localhost_signed_integration:signature-mismatch:v1"
    )

if payload["timeout_evidence_key"] != "localhost_signed_integration:timeout:v1":
    fail("timeout_evidence_key must be localhost_signed_integration:timeout:v1")

if (
    payload["success_reason_key"]
    != "localhost_signed_integration_reason:none:v1"
):
    fail(
        "success_reason_key must be "
        "localhost_signed_integration_reason:none:v1"
    )

if (
    payload["signature_mismatch_reason_key"]
    != "localhost_signed_integration_reason:signature_mismatch_detected:v1"
):
    fail(
        "signature_mismatch_reason_key must be "
        "localhost_signed_integration_reason:signature_mismatch_detected:v1"
    )

if (
    payload["timeout_reason_key"]
    != "localhost_signed_integration_reason:listener_timeout_detected:v1"
):
    fail(
        "timeout_reason_key must be "
        "localhost_signed_integration_reason:listener_timeout_detected:v1"
    )

for field in (
    "success_elapsed_seconds",
    "signature_mismatch_elapsed_seconds",
    "timeout_elapsed_seconds",
):
    value = payload[field]
    if not isinstance(value, int):
        fail(f"{field} must be an integer")
    if value < 0:
        fail(f"{field} must be non-negative")

expected_status = "pass"
if payload["success_scenario_status"] != "pass":
    expected_status = "fail"
if payload["signature_mismatch_scenario_status"] != "pass":
    expected_status = "fail"
if payload["timeout_scenario_status"] != "pass":
    expected_status = "fail"

if status != expected_status:
    fail(
        "policy status mismatch: "
        f"expected status={expected_status}, found {status}"
    )

final_decision = "GO" if status == "pass" else "NO-GO"
print("status=ok")
print(f"report_file={report_path}")
print(f"final_decision={final_decision}")
PY
)"

printf '%s\n' "$output"
