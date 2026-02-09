#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/bridge/check_localhost_bridge_demo_policy.sh \
    --bundle-file <path>
USAGE
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
    "relay",
    "replay",
    "ci_fast_gate",
    "decision_reasons",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.bridge.localhost-demo-evidence.v1":
    fail("unexpected schema_version for localhost bridge demo evidence bundle")

lane = payload["lane"]
if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")

if payload["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

if not isinstance(payload["decision_reasons"], list) or not all(
    isinstance(item, str) for item in payload["decision_reasons"]
):
    fail("decision_reasons must be an array of strings")

relay = payload["relay"]
if not isinstance(relay, dict):
    fail("relay must be an object")
for field in ("signed_transport", "relay_contracts", "completion_marker_present"):
    if field not in relay:
        fail(f"relay missing field: {field}")
if not isinstance(relay["signed_transport"], str):
    fail("relay.signed_transport must be a string")
if not isinstance(relay["relay_contracts"], str):
    fail("relay.relay_contracts must be a string")
if not isinstance(relay["completion_marker_present"], bool):
    fail("relay.completion_marker_present must be a boolean")

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

expected_go = True
if relay["signed_transport"] != "pass":
    expected_go = False
if relay["relay_contracts"] != "pass":
    expected_go = False
if relay["completion_marker_present"] is not True:
    expected_go = False
if replay["status"] != "pass":
    expected_go = False
if replay["case_count"] <= 0:
    expected_go = False
if replay["failed_count"] > 0:
    expected_go = False
if lane == "contract" and payload["ci_fast_gate"] != "PASS":
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
