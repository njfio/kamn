#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/message/check_key_lifecycle_invariant_policy.sh \
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
    "evidence_key",
    "reason_key",
    "report",
    "ci_fast_gate",
    "decision_reasons",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.key-lifecycle.invariant-evidence.v1":
    fail("unexpected schema_version for key lifecycle invariant evidence bundle")

lane = payload["lane"]
if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")

expected_evidence_key = f"key_lifecycle_invariant:{lane}:v1"
if payload["evidence_key"] != expected_evidence_key:
    fail(
        "evidence_key mismatch: "
        f"expected {expected_evidence_key}, found {payload['evidence_key']}"
    )

ci_fast_gate = payload["ci_fast_gate"]
if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

decision_reasons = payload["decision_reasons"]
if not isinstance(decision_reasons, list) or not all(
    isinstance(item, str) and item for item in decision_reasons
):
    fail("decision_reasons must be an array of non-empty strings")

report = payload["report"]
if not isinstance(report, dict):
    fail("report must be an object")
for field in (
    "status",
    "rotation_replay_detected",
    "stale_generation_detected",
    "revocation_bypass_detected",
    "reason_codes",
):
    if field not in report:
        fail(f"report missing field: {field}")

if report["status"] not in {"pass", "fail"}:
    fail("report.status must be pass or fail")
if not isinstance(report["rotation_replay_detected"], bool):
    fail("report.rotation_replay_detected must be a boolean")
if not isinstance(report["stale_generation_detected"], bool):
    fail("report.stale_generation_detected must be a boolean")
if not isinstance(report["revocation_bypass_detected"], bool):
    fail("report.revocation_bypass_detected must be a boolean")
if not isinstance(report["reason_codes"], list) or not all(
    isinstance(item, str) and item for item in report["reason_codes"]
):
    fail("report.reason_codes must be an array of non-empty strings")
if report["reason_codes"] != sorted(report["reason_codes"]):
    fail("report.reason_codes must be sorted and deterministic")

expected_go = True
if report["status"] != "pass":
    expected_go = False
if report["rotation_replay_detected"]:
    expected_go = False
if report["stale_generation_detected"]:
    expected_go = False
if report["revocation_bypass_detected"]:
    expected_go = False
if lane == "contract" and ci_fast_gate != "PASS":
    expected_go = False
if not report["reason_codes"]:
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

expected_reason_key = f"key_lifecycle_invariant_reason:{actual_decision}:v1"
if payload["reason_key"] != expected_reason_key:
    fail(
        "reason_key mismatch: "
        f"expected {expected_reason_key}, found {payload['reason_key']}"
    )

print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"schema_version={payload['schema_version']}")
print(f"evidence_key={payload['evidence_key']}")
print(f"reason_key={payload['reason_key']}")
print(f"final_decision={actual_decision}")
PY
)"

printf '%s\n' "$output"
