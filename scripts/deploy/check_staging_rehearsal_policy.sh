#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/deploy/check_staging_rehearsal_policy.sh \
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
    "rehearsal",
    "decision_reasons",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

rehearsal = payload["rehearsal"]
if not isinstance(rehearsal, dict):
    fail("bundle field 'rehearsal' must be an object")

for field in (
    "deploy_status",
    "rollback_status",
    "rollback_target_hash",
    "post_rollback_hash",
    "rollback_hash_match",
    "evidence_complete",
    "ci_fast_gate",
):
    if field not in rehearsal:
        fail(f"missing rehearsal field: {field}")

if rehearsal["deploy_status"] not in {"PASS", "FAIL"}:
    fail("rehearsal.deploy_status must be PASS or FAIL")
if rehearsal["rollback_status"] not in {"PASS", "FAIL"}:
    fail("rehearsal.rollback_status must be PASS or FAIL")
if rehearsal["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("rehearsal.ci_fast_gate must be PASS or FAIL")
if not isinstance(rehearsal["rollback_hash_match"], bool):
    fail("rehearsal.rollback_hash_match must be a boolean")
if not isinstance(rehearsal["evidence_complete"], bool):
    fail("rehearsal.evidence_complete must be a boolean")
if not isinstance(rehearsal["rollback_target_hash"], str):
    fail("rehearsal.rollback_target_hash must be a string")
if not isinstance(rehearsal["post_rollback_hash"], str):
    fail("rehearsal.post_rollback_hash must be a string")

derived_hash_match = rehearsal["rollback_target_hash"] == rehearsal["post_rollback_hash"]
if rehearsal["rollback_hash_match"] != derived_hash_match:
    fail(
        "rollback target hash mismatch: "
        f"declared rollback_hash_match={rehearsal['rollback_hash_match']} "
        f"but hashes compare as {derived_hash_match}"
    )

decision_reasons = []
if rehearsal["deploy_status"] != "PASS":
    decision_reasons.append("deploy-failed")
if rehearsal["rollback_status"] != "PASS":
    decision_reasons.append("rollback-failed")
if not rehearsal["rollback_hash_match"]:
    decision_reasons.append("rollback target hash mismatch")
if not rehearsal["evidence_complete"]:
    decision_reasons.append("incomplete evidence")
if rehearsal["ci_fast_gate"] != "PASS":
    decision_reasons.append("ci-fast-gate-failed")

expected_decision = "GO" if not decision_reasons else "NO-GO"
actual_decision = payload["final_decision"]
if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")

if actual_decision != expected_decision:
    reasons = ", ".join(decision_reasons) if decision_reasons else "all rehearsal gates satisfied"
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}; reasons={reasons}"
    )

print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"final_decision={actual_decision}")
print(f"rollback_hash_match={str(rehearsal['rollback_hash_match']).lower()}")
print(f"evidence_complete={str(rehearsal['evidence_complete']).lower()}")
PY
)"

printf '%s\n' "$output"
