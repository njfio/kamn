#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/channel/check_channel_retention_redaction_policy.sh \
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
    "retention",
    "redaction",
    "combined_reason_codes",
    "ci_fast_gate",
    "decision_reasons",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.channel.retention-redaction-evidence.v1":
    fail("unexpected schema_version for channel retention/redaction evidence bundle")

lane = payload["lane"]
if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")

expected_evidence_key = f"channel_retention_redaction:{lane}:v1"
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

retention = payload["retention"]
if not isinstance(retention, dict):
    fail("retention must be an object")
for field in ("status", "total_candidates", "replay_safe", "reason_codes"):
    if field not in retention:
        fail(f"retention missing field: {field}")
if retention["status"] not in {"pass", "fail"}:
    fail("retention.status must be pass or fail")
if not isinstance(retention["total_candidates"], int):
    fail("retention.total_candidates must be an integer")
if not isinstance(retention["replay_safe"], bool):
    fail("retention.replay_safe must be a boolean")
if not isinstance(retention["reason_codes"], list):
    fail("retention.reason_codes must be an array")
if retention["reason_codes"] != sorted(retention["reason_codes"]):
    fail("retention.reason_codes must be sorted and deterministic")
if not all(isinstance(item, str) and item for item in retention["reason_codes"]):
    fail("retention.reason_codes must contain non-empty strings")

redaction = payload["redaction"]
if not isinstance(redaction, dict):
    fail("redaction must be an object")
for field in ("status", "applied_count", "replay_safe", "reason_codes"):
    if field not in redaction:
        fail(f"redaction missing field: {field}")
if redaction["status"] not in {"pass", "fail"}:
    fail("redaction.status must be pass or fail")
if not isinstance(redaction["applied_count"], int):
    fail("redaction.applied_count must be an integer")
if not isinstance(redaction["replay_safe"], bool):
    fail("redaction.replay_safe must be a boolean")
if not isinstance(redaction["reason_codes"], list):
    fail("redaction.reason_codes must be an array")
if redaction["reason_codes"] != sorted(redaction["reason_codes"]):
    fail("redaction.reason_codes must be sorted and deterministic")
if not all(isinstance(item, str) and item for item in redaction["reason_codes"]):
    fail("redaction.reason_codes must contain non-empty strings")

combined_reason_codes = payload["combined_reason_codes"]
if not isinstance(combined_reason_codes, list) or not all(
    isinstance(item, str) and item for item in combined_reason_codes
):
    fail("combined_reason_codes must be an array of non-empty strings")
if combined_reason_codes != sorted(set(combined_reason_codes)):
    fail("combined_reason_codes must be sorted unique deterministic values")

expected_go = True
if retention["status"] != "pass":
    expected_go = False
if redaction["status"] != "pass":
    expected_go = False
if not retention["replay_safe"]:
    expected_go = False
if not redaction["replay_safe"]:
    expected_go = False
if lane == "contract" and ci_fast_gate != "PASS":
    expected_go = False
if not retention["reason_codes"]:
    expected_go = False
if not redaction["reason_codes"]:
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

expected_reason_key = f"channel_retention_redaction_reason:{actual_decision}:v1"
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
