#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/deploy/generate_dr_evidence_bundle.sh \
    --output-file <path> \
    --drill-id <value> \
    --recovery-rto-seconds <n> \
    --recovery-rpo-seconds <n> \
    --max-rto-seconds <n> \
    --max-rpo-seconds <n> \
    --rollback-restored true|false \
    --evidence-complete true|false \
    --ci-fast-gate PASS|FAIL
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

require_int() {
  local name="$1"
  local value="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    fail "${name} must be an integer"
  fi
}

output_file=""
drill_id=""
recovery_rto_seconds=""
recovery_rpo_seconds=""
max_rto_seconds=""
max_rpo_seconds=""
rollback_restored=""
evidence_complete=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --drill-id)
      drill_id="${2:-}"
      shift 2
      ;;
    --recovery-rto-seconds)
      recovery_rto_seconds="${2:-}"
      shift 2
      ;;
    --recovery-rpo-seconds)
      recovery_rpo_seconds="${2:-}"
      shift 2
      ;;
    --max-rto-seconds)
      max_rto_seconds="${2:-}"
      shift 2
      ;;
    --max-rpo-seconds)
      max_rpo_seconds="${2:-}"
      shift 2
      ;;
    --rollback-restored)
      rollback_restored="${2:-}"
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

if [[ -z "$output_file" || -z "$drill_id" || -z "$recovery_rto_seconds" || -z "$recovery_rpo_seconds" || -z "$max_rto_seconds" || -z "$max_rpo_seconds" || -z "$rollback_restored" || -z "$evidence_complete" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all DR evidence bundle arguments are required"
fi

require_int "recovery-rto-seconds" "$recovery_rto_seconds"
require_int "recovery-rpo-seconds" "$recovery_rpo_seconds"
require_int "max-rto-seconds" "$max_rto_seconds"
require_int "max-rpo-seconds" "$max_rpo_seconds"

if (( max_rto_seconds < 1 )); then
  fail "max-rto-seconds must be >= 1"
fi
if (( max_rpo_seconds < 1 )); then
  fail "max-rpo-seconds must be >= 1"
fi

mkdir -p "$(dirname "$output_file")"

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$drill_id" "$recovery_rto_seconds" "$recovery_rpo_seconds" "$max_rto_seconds" "$max_rpo_seconds" "$rollback_restored" "$evidence_complete" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys


def fail(message: str) -> None:
    raise ValueError(message)


(
    output_file,
    generated_at,
    drill_id,
    recovery_rto_raw,
    recovery_rpo_raw,
    max_rto_raw,
    max_rpo_raw,
    rollback_restored_raw,
    evidence_complete_raw,
    ci_fast_gate,
) = sys.argv[1:]

if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci-fast-gate must be PASS or FAIL")

if rollback_restored_raw not in {"true", "false"}:
    fail("rollback-restored must be true or false")
if evidence_complete_raw not in {"true", "false"}:
    fail("evidence-complete must be true or false")

rollback_restored = rollback_restored_raw == "true"
evidence_complete = evidence_complete_raw == "true"

recovery_rto = int(recovery_rto_raw)
recovery_rpo = int(recovery_rpo_raw)
max_rto = int(max_rto_raw)
max_rpo = int(max_rpo_raw)

decision_reasons = []
if recovery_rto > max_rto:
    decision_reasons.append("rto threshold exceeded")
if recovery_rpo > max_rpo:
    decision_reasons.append("rpo threshold exceeded")
if not rollback_restored:
    decision_reasons.append("rollback not restored")
if not evidence_complete:
    decision_reasons.append("incomplete drill evidence")
if ci_fast_gate != "PASS":
    decision_reasons.append("ci-fast-gate-failed")

final_decision = "GO" if not decision_reasons else "NO-GO"
if not decision_reasons:
    decision_reasons.append("all dr evidence gates satisfied")

payload = {
    "schema_version": "kamn.release.dr-evidence.v1",
    "generated_at": generated_at,
    "drill_id": drill_id,
    "dr_evidence": {
        "recovery_rto_seconds": recovery_rto,
        "recovery_rpo_seconds": recovery_rpo,
        "max_rto_seconds": max_rto,
        "max_rpo_seconds": max_rpo,
        "rollback_restored": rollback_restored,
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
