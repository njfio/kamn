#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/runtime/check_invariant_fuzz_concurrency_policy.sh \
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
    "property_lane_status",
    "fuzz_lane_status",
    "concurrency_lane_status",
    "property_replay_schema_version",
    "property_replay_artifact_key",
    "property_replay_test_count",
    "elapsed_seconds",
    "max_seconds",
    "reason_codes",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing report field: {field}")

if payload["schema_version"] != "kamn.runtime.invariant-fuzz-concurrency-contract-report.v1":
    fail("unexpected schema_version for invariant/fuzz/concurrency report")

status = payload["status"]
if status not in {"pass", "fail"}:
    fail("status must be pass or fail")

for field in ("property_lane_status", "fuzz_lane_status", "concurrency_lane_status"):
    if payload[field] not in {"pass", "fail"}:
        fail(f"{field} must be pass or fail")

property_replay_schema_version = payload["property_replay_schema_version"]
expected_property_replay_schema_version = "kamn.runtime.lifecycle-property-contract-report.v1"
if property_replay_schema_version != expected_property_replay_schema_version:
    fail(
        "property_replay_schema_version mismatch: "
        f"expected {expected_property_replay_schema_version}, found {property_replay_schema_version}"
    )

property_replay_artifact_key = payload["property_replay_artifact_key"]
expected_property_replay_artifact_key = "lifecycle_property_replay:v1"
if property_replay_artifact_key != expected_property_replay_artifact_key:
    fail(
        "property_replay_artifact_key mismatch: "
        f"expected {expected_property_replay_artifact_key}, found {property_replay_artifact_key}"
    )

property_replay_test_count = payload["property_replay_test_count"]
if not isinstance(property_replay_test_count, int) or property_replay_test_count < 12:
    fail("property_replay_test_count must be an integer >= 12")

elapsed_seconds = payload["elapsed_seconds"]
max_seconds = payload["max_seconds"]
if not isinstance(elapsed_seconds, int) or elapsed_seconds < 0:
    fail("elapsed_seconds must be a non-negative integer")
if not isinstance(max_seconds, int) or max_seconds <= 0:
    fail("max_seconds must be a positive integer")

reason_codes = payload["reason_codes"]
if not isinstance(reason_codes, list) or not reason_codes:
    fail("reason_codes must be a non-empty list")
if not all(isinstance(item, str) and item for item in reason_codes):
    fail("reason_codes must contain non-empty strings")

expected_status = "pass"
expected_reason_codes = ["none"]
if (
    payload["property_lane_status"] != "pass"
    or payload["fuzz_lane_status"] != "pass"
    or payload["concurrency_lane_status"] != "pass"
    or elapsed_seconds > max_seconds
):
    expected_status = "fail"
    expected_reason_codes = ["lane_failure_or_runtime_budget_exceeded"]

if status != expected_status:
    fail(
        "policy status mismatch: "
        f"expected status={expected_status}, found {status}"
    )

if reason_codes != expected_reason_codes:
    fail(
        "reason_codes mismatch: "
        f"expected {expected_reason_codes}, found {reason_codes}"
    )

final_decision = "GO" if status == "pass" else "NO-GO"
print("status=ok")
print(f"report_file={report_path}")
print(f"final_decision={final_decision}")
PY
)"

printf '%s\n' "$output"
