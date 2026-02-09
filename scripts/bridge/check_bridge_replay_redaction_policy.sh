#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/bridge/check_bridge_replay_redaction_policy.sh \
    --bundle-file <path>
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

bundle_file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bundle-file)
      bundle_file="${2:-}"
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

if [[ -z "$bundle_file" ]]; then
  usage
  fail "--bundle-file is required"
fi

if [[ ! -f "$bundle_file" ]]; then
  fail "bundle file not found: $bundle_file"
fi

output="$(
  python3 - "$bundle_file" <<'PY'
import json
import pathlib
import sys


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    sys.exit(1)


bundle_path = pathlib.Path(sys.argv[1])
try:
    payload = json.loads(bundle_path.read_text())
except json.JSONDecodeError as exc:
    fail(f"bundle file is not valid JSON: {exc}")

required_fields = (
    "schema_version",
    "generated_at",
    "lane",
    "replay",
    "redaction",
    "ci_fast_gate",
    "decision_reasons",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.bridge.replay-redaction-evidence.v1":
    fail("unexpected schema_version for bridge replay/redaction evidence bundle")

lane = payload["lane"]
if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")

if payload["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

if not isinstance(payload["decision_reasons"], list) or not all(
    isinstance(item, str) for item in payload["decision_reasons"]
):
    fail("decision_reasons must be an array of strings")

replay = payload["replay"]
if not isinstance(replay, dict):
    fail("replay must be an object")
for field in ("status", "case_count", "failed_count", "requested_suites", "failed_case_ids"):
    if field not in replay:
        fail(f"replay missing field: {field}")
if replay["status"] not in {"pass", "fail"}:
    fail("replay.status must be pass or fail")
if not isinstance(replay["case_count"], int):
    fail("replay.case_count must be an integer")
if not isinstance(replay["failed_count"], int):
    fail("replay.failed_count must be an integer")
if not isinstance(replay["requested_suites"], list):
    fail("replay.requested_suites must be an array")
if not isinstance(replay["failed_case_ids"], list):
    fail("replay.failed_case_ids must be an array")

redaction = payload["redaction"]
if not isinstance(redaction, dict):
    fail("redaction must be an object")
for field in (
    "status",
    "mode",
    "connector_count",
    "leaked_connector_count",
    "leaked_connectors",
):
    if field not in redaction:
        fail(f"redaction missing field: {field}")
if redaction["status"] not in {"pass", "fail"}:
    fail("redaction.status must be pass or fail")
if redaction["mode"] not in {"contract", "deep"}:
    fail("redaction.mode must be contract or deep")
if not isinstance(redaction["connector_count"], int):
    fail("redaction.connector_count must be an integer")
if not isinstance(redaction["leaked_connector_count"], int):
    fail("redaction.leaked_connector_count must be an integer")
if not isinstance(redaction["leaked_connectors"], list):
    fail("redaction.leaked_connectors must be an array")

expected_go = True
if replay["status"] != "pass":
    expected_go = False
if replay["case_count"] <= 0:
    expected_go = False
if replay["failed_count"] > 0:
    expected_go = False
if redaction["status"] != "pass":
    expected_go = False
if redaction["connector_count"] <= 0:
    expected_go = False
if redaction["leaked_connector_count"] > 0:
    expected_go = False
if lane == "contract" and payload["ci_fast_gate"] != "PASS":
    expected_go = False
if lane == "contract" and redaction["mode"] != "contract":
    expected_go = False
if lane == "deep" and redaction["mode"] != "deep":
    expected_go = False

expected_decision = "GO" if expected_go else "NO-GO"
actual_decision = payload["final_decision"]
if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")
if actual_decision != expected_decision:
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}"
    )

print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"final_decision={actual_decision}")
PY
)"

printf '%s\n' "$output"
