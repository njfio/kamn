#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/deploy/check_gonogo_evidence_policy.sh \
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
    "release_candidate",
    "schema_target_version",
    "runtime_image_digest",
    "gates",
    "rollback_trigger_status",
    "approvals",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

gates = payload["gates"]
if not isinstance(gates, dict):
    fail("bundle field 'gates' must be an object")

for gate_name in ("ci_fast_gate", "ci_deep_lane", "rollback_precheck"):
    if gate_name not in gates:
        fail(f"missing gate field: {gate_name}")
    if gates[gate_name] not in {"PASS", "FAIL"}:
        fail(f"gate '{gate_name}' must be PASS or FAIL")

rollback_trigger_status = payload["rollback_trigger_status"]
if rollback_trigger_status not in {"CLEAR", "TRIGGERED"}:
    fail("rollback_trigger_status must be CLEAR or TRIGGERED")

approvals = payload["approvals"]
if not isinstance(approvals, dict):
    fail("bundle field 'approvals' must be an object")

if "required" not in approvals:
    fail("missing approvals field: required")
if "received" not in approvals:
    fail("missing approvals field: received")

required = approvals["required"]
received = approvals["received"]
if not isinstance(required, int):
    fail("approvals.required must be an integer")
if not isinstance(received, int):
    fail("approvals.received must be an integer")
if required < 1:
    fail("approvals.required must be >= 1")
if received < 0:
    fail("approvals.received must be >= 0")

expected_go = (
    gates["ci_fast_gate"] == "PASS"
    and gates["ci_deep_lane"] == "PASS"
    and gates["rollback_precheck"] == "PASS"
    and rollback_trigger_status == "CLEAR"
    and received >= required
)
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
print(f"required_approvals={required}")
print(f"received_approvals={received}")
PY
)"

printf '%s\n' "$output"
