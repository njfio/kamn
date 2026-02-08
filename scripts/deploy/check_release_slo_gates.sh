#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/deploy/check_release_slo_gates.sh \
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
    "drill_id",
    "dr_evidence",
    "decision_reasons",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

dr_evidence = payload["dr_evidence"]
if not isinstance(dr_evidence, dict):
    fail("bundle field 'dr_evidence' must be an object")

for field in (
    "recovery_rto_seconds",
    "recovery_rpo_seconds",
    "max_rto_seconds",
    "max_rpo_seconds",
    "rollback_restored",
    "evidence_complete",
    "ci_fast_gate",
):
    if field not in dr_evidence:
        fail(f"missing dr_evidence field: {field}")

for numeric_field in (
    "recovery_rto_seconds",
    "recovery_rpo_seconds",
    "max_rto_seconds",
    "max_rpo_seconds",
):
    if not isinstance(dr_evidence[numeric_field], int):
        fail(f"dr_evidence.{numeric_field} must be an integer")

if dr_evidence["max_rto_seconds"] < 1:
    fail("dr_evidence.max_rto_seconds must be >= 1")
if dr_evidence["max_rpo_seconds"] < 1:
    fail("dr_evidence.max_rpo_seconds must be >= 1")

if not isinstance(dr_evidence["rollback_restored"], bool):
    fail("dr_evidence.rollback_restored must be a boolean")
if not isinstance(dr_evidence["evidence_complete"], bool):
    fail("dr_evidence.evidence_complete must be a boolean")
if dr_evidence["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("dr_evidence.ci_fast_gate must be PASS or FAIL")

decision_reasons = []
if dr_evidence["recovery_rto_seconds"] > dr_evidence["max_rto_seconds"]:
    decision_reasons.append("rto threshold exceeded")
if dr_evidence["recovery_rpo_seconds"] > dr_evidence["max_rpo_seconds"]:
    decision_reasons.append("rpo threshold exceeded")
if not dr_evidence["rollback_restored"]:
    decision_reasons.append("rollback not restored")
if not dr_evidence["evidence_complete"]:
    decision_reasons.append("incomplete drill evidence")
if dr_evidence["ci_fast_gate"] != "PASS":
    decision_reasons.append("ci-fast-gate-failed")

expected_decision = "GO" if not decision_reasons else "NO-GO"
actual_decision = payload["final_decision"]
if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")

if actual_decision != expected_decision:
    reasons = ", ".join(decision_reasons) if decision_reasons else "all dr evidence gates satisfied"
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}; reasons={reasons}"
    )

print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"final_decision={actual_decision}")
print(f"recovery_rto_seconds={dr_evidence['recovery_rto_seconds']}")
print(f"recovery_rpo_seconds={dr_evidence['recovery_rpo_seconds']}")
PY
)"

printf '%s\n' "$output"
