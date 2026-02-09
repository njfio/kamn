#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/cutover/check_cutover_rollback_evidence_policy.sh \
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
    "cutover_manifest_id",
    "rollback",
    "decision_reasons",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.cutover.rollback-evidence.v1":
    fail("unexpected rollback evidence schema_version")

if not isinstance(payload["cutover_manifest_id"], str) or not payload["cutover_manifest_id"].strip():
    fail("cutover_manifest_id must be a non-empty string")

rollback = payload["rollback"]
if not isinstance(rollback, dict):
    fail("bundle field 'rollback' must be an object")

for field in (
    "trigger_status",
    "checkpoint_state",
    "failed_checkpoint_id",
    "rollback_target_hash",
    "post_rollback_hash",
    "rollback_hash_match",
    "evidence_complete",
    "ci_fast_gate",
):
    if field not in rollback:
        fail(f"missing rollback field: {field}")

if rollback["trigger_status"] not in {"CLEAR", "TRIGGERED"}:
    fail("rollback.trigger_status must be CLEAR or TRIGGERED")
if rollback["checkpoint_state"] not in {"READY", "FAILED"}:
    fail("rollback.checkpoint_state must be READY or FAILED")
if rollback["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("rollback.ci_fast_gate must be PASS or FAIL")
if rollback["failed_checkpoint_id"] is not None and not isinstance(rollback["failed_checkpoint_id"], str):
    fail("rollback.failed_checkpoint_id must be a string or null")
if not isinstance(rollback["rollback_target_hash"], str):
    fail("rollback.rollback_target_hash must be a string")
if not isinstance(rollback["post_rollback_hash"], str):
    fail("rollback.post_rollback_hash must be a string")
if not isinstance(rollback["rollback_hash_match"], bool):
    fail("rollback.rollback_hash_match must be a boolean")
if not isinstance(rollback["evidence_complete"], bool):
    fail("rollback.evidence_complete must be a boolean")

derived_hash_match = rollback["rollback_target_hash"] == rollback["post_rollback_hash"]
if rollback["rollback_hash_match"] != derived_hash_match:
    fail(
        "rollback target hash mismatch: "
        f"declared rollback_hash_match={rollback['rollback_hash_match']} "
        f"but hashes compare as {derived_hash_match}"
    )

decision_reasons: list[str] = []
if rollback["ci_fast_gate"] != "PASS":
    decision_reasons.append("ci-fast-gate-failed")
if not rollback["evidence_complete"]:
    decision_reasons.append("incomplete-evidence")
if not rollback["rollback_hash_match"]:
    decision_reasons.append("rollback target hash mismatch")
if rollback["trigger_status"] == "TRIGGERED" and not rollback["failed_checkpoint_id"]:
    decision_reasons.append("missing failed checkpoint evidence")
if rollback["trigger_status"] == "TRIGGERED" and rollback["checkpoint_state"] != "FAILED":
    decision_reasons.append("trigger-state-checkpoint-mismatch")
if rollback["trigger_status"] == "CLEAR" and rollback["checkpoint_state"] != "READY":
    decision_reasons.append("clear-trigger-requires-ready-checkpoint")

expected_decision = "GO" if not decision_reasons else "NO-GO"
actual_decision = payload["final_decision"]
if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")

if actual_decision != expected_decision:
    reasons = ", ".join(decision_reasons) if decision_reasons else "all rollback gates satisfied"
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}; reasons={reasons}"
    )

print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"final_decision={actual_decision}")
print(f"trigger_status={rollback['trigger_status']}")
print(f"rollback_hash_match={str(rollback['rollback_hash_match']).lower()}")
print(f"evidence_complete={str(rollback['evidence_complete']).lower()}")
PY
)"

printf '%s\n' "$output"
