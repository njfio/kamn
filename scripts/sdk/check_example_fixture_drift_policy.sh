#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_SCHEMA="kamn.sdk.example-fixture-drift-report.v1"

usage() {
  cat <<'EOF'
Usage:
  bash scripts/sdk/check_example_fixture_drift_policy.sh --report-file <path>
EOF
}

fail() {
  local reason="$1"
  echo "status=fail"
  echo "reason=$reason"
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
      fail "unknown-argument:$1"
      ;;
  esac
done

if [ -z "$report_file" ]; then
  fail "missing-report-file"
fi

if [ ! -f "$report_file" ]; then
  fail "report-file-not-found"
fi

set +e
validation_output="$(
  python3 - "$report_file" "$REPORT_SCHEMA" <<'PY'
import json
import pathlib
import sys

report_file = pathlib.Path(sys.argv[1])
expected_schema = sys.argv[2]
payload = json.loads(report_file.read_text(encoding="utf-8"))

if payload.get("schema_version") != expected_schema:
    print("status=fail")
    print("reason=invalid-schema-version")
    raise SystemExit(1)

status = payload.get("status")
if status not in {"pass", "fail"}:
    print("status=fail")
    print("reason=invalid-status")
    raise SystemExit(1)

reason_codes = payload.get("reason_codes")
if not isinstance(reason_codes, list) or not reason_codes:
    print("status=fail")
    print("reason=invalid-reason-codes")
    raise SystemExit(1)

if status == "pass":
    if reason_codes != ["none"]:
        print("status=fail")
        print("reason=unexpected-pass-reason-codes")
        raise SystemExit(1)
    print("status=ok")
    print("final_decision=GO")
    print("reason_codes=none")
    raise SystemExit(0)

if "none" in reason_codes:
    print("status=fail")
    print("reason=unexpected-none-reason-code")
    raise SystemExit(1)

print("status=ok")
print("final_decision=NO-GO")
print(f"reason_codes={','.join(reason_codes)}")
raise SystemExit(0)
PY
)"
validation_status=$?
set -e

printf '%s\n' "$validation_output"
if [ "$validation_status" -ne 0 ]; then
  exit "$validation_status"
fi
