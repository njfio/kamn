#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/deploy/generate_staging_rehearsal_bundle.sh \
    --output-file <path> \
    --release-candidate <value> \
    --deploy-status PASS|FAIL \
    --rollback-status PASS|FAIL \
    --rollback-target-hash <value> \
    --post-rollback-hash <value> \
    --evidence-complete true|false \
    --ci-fast-gate PASS|FAIL
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

output_file=""
release_candidate=""
deploy_status=""
rollback_status=""
rollback_target_hash=""
post_rollback_hash=""
evidence_complete=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --release-candidate)
      release_candidate="${2:-}"
      shift 2
      ;;
    --deploy-status)
      deploy_status="${2:-}"
      shift 2
      ;;
    --rollback-status)
      rollback_status="${2:-}"
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
      evidence_complete="${2:-}"
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

if [[ -z "$output_file" || -z "$release_candidate" || -z "$deploy_status" || -z "$rollback_status" || -z "$rollback_target_hash" || -z "$post_rollback_hash" || -z "$evidence_complete" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all rehearsal bundle arguments are required"
fi

mkdir -p "$(dirname "$output_file")"

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$release_candidate" "$deploy_status" "$rollback_status" "$rollback_target_hash" "$post_rollback_hash" "$evidence_complete" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys


def fail(message: str) -> None:
    raise ValueError(message)


(
    output_file,
    generated_at,
    release_candidate,
    deploy_status,
    rollback_status,
    rollback_target_hash,
    post_rollback_hash,
    evidence_complete_raw,
    ci_fast_gate,
) = sys.argv[1:]

if deploy_status not in {"PASS", "FAIL"}:
    fail("deploy-status must be PASS or FAIL")
if rollback_status not in {"PASS", "FAIL"}:
    fail("rollback-status must be PASS or FAIL")
if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci-fast-gate must be PASS or FAIL")

if evidence_complete_raw not in {"true", "false"}:
    fail("evidence-complete must be true or false")
evidence_complete = evidence_complete_raw == "true"

rollback_hash_match = rollback_target_hash == post_rollback_hash
decision_reasons = []
if deploy_status != "PASS":
    decision_reasons.append("deploy-failed")
if rollback_status != "PASS":
    decision_reasons.append("rollback-failed")
if not rollback_hash_match:
    decision_reasons.append("rollback target hash mismatch")
if not evidence_complete:
    decision_reasons.append("incomplete evidence")
if ci_fast_gate != "PASS":
    decision_reasons.append("ci-fast-gate-failed")

final_decision = "GO" if not decision_reasons else "NO-GO"
if not decision_reasons:
    decision_reasons.append("all rehearsal gates satisfied")

payload = {
    "schema_version": "kamn.release.staging-rehearsal.v1",
    "generated_at": generated_at,
    "release_candidate": release_candidate,
    "rehearsal": {
        "deploy_status": deploy_status,
        "rollback_status": rollback_status,
        "rollback_target_hash": rollback_target_hash,
        "post_rollback_hash": post_rollback_hash,
        "rollback_hash_match": rollback_hash_match,
        "evidence_complete": evidence_complete,
        "ci_fast_gate": ci_fast_gate,
    },
    "decision_reasons": decision_reasons,
    "final_decision": final_decision,
}

path = pathlib.Path(output_file)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
print(final_decision)
PY
)"

printf 'status=generated\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'final_decision=%s\n' "$final_decision"
