#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/cutover/generate_cutover_rollback_evidence_bundle.sh \
    --output-file <path> \
    --cutover-manifest-id <id> \
    --rollback-trigger-status <CLEAR|TRIGGERED> \
    --checkpoint-state <READY|FAILED> \
    --failed-checkpoint-id <id-or-empty> \
    --rollback-target-hash <hash> \
    --post-rollback-hash <hash> \
    --evidence-complete <true|false> \
    --ci-fast-gate <PASS|FAIL>
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

to_bool() {
  local field="$1"
  local value="$2"
  case "$value" in
    true|false)
      printf '%s' "$value"
      ;;
    *)
      fail "${field} must be true or false"
      ;;
  esac
}

output_file=""
cutover_manifest_id=""
rollback_trigger_status=""
checkpoint_state=""
failed_checkpoint_id=""
rollback_target_hash=""
post_rollback_hash=""
evidence_complete_raw=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --cutover-manifest-id)
      cutover_manifest_id="${2:-}"
      shift 2
      ;;
    --rollback-trigger-status)
      rollback_trigger_status="${2:-}"
      shift 2
      ;;
    --checkpoint-state)
      checkpoint_state="${2:-}"
      shift 2
      ;;
    --failed-checkpoint-id)
      failed_checkpoint_id="${2:-}"
      shift 2
      ;;
    --rollback-target-hash)
      rollback_target_hash="${2:-}"
      shift 2
      ;;
    --post-rollback-hash)
      post_rollback_hash="${2:-}"
      shift 2
      ;;
    --evidence-complete)
      evidence_complete_raw="${2:-}"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
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

if [[ -z "$output_file" || -z "$cutover_manifest_id" || -z "$rollback_trigger_status" || -z "$checkpoint_state" || -z "$rollback_target_hash" || -z "$post_rollback_hash" || -z "$evidence_complete_raw" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all generator arguments are required"
fi

case "$rollback_trigger_status" in
  CLEAR|TRIGGERED) ;;
  *)
    fail "--rollback-trigger-status must be CLEAR or TRIGGERED"
    ;;
esac

case "$checkpoint_state" in
  READY|FAILED) ;;
  *)
    fail "--checkpoint-state must be READY or FAILED"
    ;;
esac

case "$ci_fast_gate" in
  PASS|FAIL) ;;
  *)
    fail "--ci-fast-gate must be PASS or FAIL"
    ;;
esac

evidence_complete="$(to_bool "evidence_complete" "$evidence_complete_raw")"
mkdir -p "$(dirname "$output_file")"

python3 - "$output_file" "$cutover_manifest_id" "$rollback_trigger_status" "$checkpoint_state" "$failed_checkpoint_id" "$rollback_target_hash" "$post_rollback_hash" "$evidence_complete" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys
from datetime import datetime, timezone

output_file = pathlib.Path(sys.argv[1])
cutover_manifest_id = sys.argv[2]
rollback_trigger_status = sys.argv[3]
checkpoint_state = sys.argv[4]
failed_checkpoint_id = sys.argv[5]
rollback_target_hash = sys.argv[6]
post_rollback_hash = sys.argv[7]
evidence_complete = sys.argv[8] == "true"
ci_fast_gate = sys.argv[9]

rollback_hash_match = rollback_target_hash == post_rollback_hash

decision_reasons: list[str] = []
if ci_fast_gate != "PASS":
    decision_reasons.append("ci-fast-gate-failed")
if not evidence_complete:
    decision_reasons.append("incomplete-evidence")
if not rollback_hash_match:
    decision_reasons.append("rollback target hash mismatch")
if rollback_trigger_status == "TRIGGERED" and not failed_checkpoint_id:
    decision_reasons.append("missing failed checkpoint evidence")
if rollback_trigger_status == "TRIGGERED" and checkpoint_state != "FAILED":
    decision_reasons.append("trigger-state-checkpoint-mismatch")
if rollback_trigger_status == "CLEAR" and checkpoint_state != "READY":
    decision_reasons.append("clear-trigger-requires-ready-checkpoint")

final_decision = "GO" if not decision_reasons else "NO-GO"

payload = {
    "schema_version": "kamn.cutover.rollback-evidence.v1",
    "generated_at": datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "cutover_manifest_id": cutover_manifest_id,
    "rollback": {
        "trigger_status": rollback_trigger_status,
        "checkpoint_state": checkpoint_state,
        "failed_checkpoint_id": failed_checkpoint_id if failed_checkpoint_id else None,
        "rollback_target_hash": rollback_target_hash,
        "post_rollback_hash": post_rollback_hash,
        "rollback_hash_match": rollback_hash_match,
        "evidence_complete": evidence_complete,
        "ci_fast_gate": ci_fast_gate,
    },
    "decision_reasons": decision_reasons,
    "final_decision": final_decision,
}
output_file.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
print("status=generated")
print(f"bundle_file={output_file}")
print(f"final_decision={final_decision}")
print(f"rollback_hash_match={str(rollback_hash_match).lower()}")
print(f"evidence_complete={str(evidence_complete).lower()}")
PY
