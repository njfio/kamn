#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/sdk/check_live_transport_smoke_parity_policy.sh \
    --report-file <path>
EOF
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
from typing import List


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    sys.exit(1)


path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))

required_fields = (
    "schema_version",
    "status",
    "final_decision",
    "elapsed_seconds",
    "max_seconds",
    "max_retries",
    "retry_attempts",
    "retry_used",
    "retry_final_status",
    "languages",
    "skip_commands",
    "force_failure",
    "command_count",
    "commands",
    "reason_codes",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing field: {field}")

if payload["schema_version"] != "kamn.sdk.live-transport-smoke-parity-report.v1":
    fail("unexpected schema_version for sdk live transport smoke parity report")

status = payload["status"]
if status not in {"pass", "fail"}:
    fail("status must be pass or fail")

final_decision = payload["final_decision"]
if final_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")

if not isinstance(payload["elapsed_seconds"], int):
    fail("elapsed_seconds must be an integer")
if not isinstance(payload["max_seconds"], int):
    fail("max_seconds must be an integer")
if payload["max_seconds"] < 0:
    fail("max_seconds must be non-negative")

if not isinstance(payload["max_retries"], int):
    fail("max_retries must be an integer")
if payload["max_retries"] < 0 or payload["max_retries"] > 2:
    fail("max_retries must be between 0 and 2")

if not isinstance(payload["retry_attempts"], int):
    fail("retry_attempts must be an integer")
if payload["retry_attempts"] < 1:
    fail("retry_attempts must be at least 1")

retry_used = payload["retry_used"]
if not isinstance(retry_used, bool):
    fail("retry_used must be boolean")

retry_final_status = payload["retry_final_status"]
if retry_final_status not in {"passed", "failed"}:
    fail("retry_final_status must be passed or failed")

languages = payload["languages"]
if not isinstance(languages, list) or not languages:
    fail("languages must be a non-empty array")
if not all(isinstance(item, str) and item in {"rust", "python", "typescript"} for item in languages):
    fail("languages must contain only rust/python/typescript values")

if not isinstance(payload["skip_commands"], bool):
    fail("skip_commands must be boolean")
if not isinstance(payload["force_failure"], bool):
    fail("force_failure must be boolean")

commands = payload["commands"]
if not isinstance(commands, list):
    fail("commands must be an array")
if not all(isinstance(item, str) and item for item in commands):
    fail("commands must contain non-empty command strings")

command_count = payload["command_count"]
if not isinstance(command_count, int):
    fail("command_count must be an integer")
if command_count != len(commands):
    fail("command_count must match commands array length")

reason_codes = payload["reason_codes"]
if not isinstance(reason_codes, list):
    fail("reason_codes must be an array")
if not all(isinstance(item, str) and item for item in reason_codes):
    fail("reason_codes must contain non-empty strings")
if reason_codes != sorted(reason_codes):
    fail("reason_codes must be sorted and deterministic")

max_attempts = payload["max_retries"] + 1
retry_attempts = payload["retry_attempts"]
if retry_attempts > max_attempts:
    fail("retry_attempts exceeds retry budget")
if not retry_used and retry_attempts > 1:
    fail("retry_used=false is invalid when retry_attempts > 1")

expected_reasons: List[str] = []
if retry_final_status != "passed":
    expected_reasons.append("smoke_lane_failed")
if retry_final_status != "passed" and retry_used:
    expected_reasons.append("retry_budget_exceeded")
if payload["elapsed_seconds"] > payload["max_seconds"]:
    expected_reasons.append("runtime_budget_exceeded")
expected_reasons = sorted(expected_reasons)

expected_status = "pass" if not expected_reasons else "fail"
if status != expected_status:
    fail(f"status mismatch: expected {expected_status}, found {status}")

expected_decision = "GO" if not expected_reasons else "NO-GO"
if final_decision != expected_decision:
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {final_decision}"
    )

if reason_codes != expected_reasons:
    fail(
        "reason_codes mismatch: "
        f"expected reason_codes={expected_reasons}, found {reason_codes}"
    )

failed_checks = ",".join(expected_reasons) if expected_reasons else "none"
print("status=ok")
print(f"report_file={path}")
print(f"final_decision={final_decision}")
print(f"failed_checks={failed_checks}")
PY
)"

printf '%s\n' "$output"
