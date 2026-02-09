#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/channel/generate_channel_retention_redaction_evidence_bundle.sh \
    --output-file <path> \
    --lane contract|deep \
    --retention-report-file <path> \
    --redaction-report-file <path> \
    --ci-fast-gate PASS|FAIL
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

output_file=""
lane=""
retention_report_file=""
redaction_report_file=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --lane)
      lane="${2:-}"
      shift 2
      ;;
    --retention-report-file)
      retention_report_file="${2:-}"
      shift 2
      ;;
    --redaction-report-file)
      redaction_report_file="${2:-}"
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

if [[ -z "$output_file" || -z "$lane" || -z "$retention_report_file" || -z "$redaction_report_file" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all channel retention/redaction evidence bundle arguments are required"
fi

if [[ ! -f "$retention_report_file" ]]; then
  fail "retention report file not found: $retention_report_file"
fi

if [[ ! -f "$redaction_report_file" ]]; then
  fail "redaction report file not found: $redaction_report_file"
fi

mkdir -p "$(dirname "$output_file")"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$lane" "$retention_report_file" "$redaction_report_file" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys


def fail(message: str) -> None:
    raise ValueError(message)


(
    output_file,
    generated_at,
    lane,
    retention_report_file,
    redaction_report_file,
    ci_fast_gate,
) = sys.argv[1:]

if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")
if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci-fast-gate must be PASS or FAIL")

retention_payload = json.loads(pathlib.Path(retention_report_file).read_text())
redaction_payload = json.loads(pathlib.Path(redaction_report_file).read_text())


def parse_reason_codes(payload: dict, field_prefix: str) -> list[str]:
    reason_codes = payload.get("reason_codes", [])
    if not isinstance(reason_codes, list):
        fail(f"{field_prefix}.reason_codes must be an array")
    if not all(isinstance(item, str) and item for item in reason_codes):
        fail(f"{field_prefix}.reason_codes must contain non-empty strings")
    # Deterministic ordering for replay-safe policy checks.
    return sorted(reason_codes)


retention_status = str(retention_payload.get("status", ""))
retention_total_candidates = retention_payload.get("total_candidates")
retention_replay_safe = retention_payload.get("replay_safe")
retention_reason_codes = parse_reason_codes(retention_payload, "retention")

if retention_status not in {"pass", "fail"}:
    fail("retention.status must be pass or fail")
if not isinstance(retention_total_candidates, int):
    fail("retention.total_candidates must be an integer")
if not isinstance(retention_replay_safe, bool):
    fail("retention.replay_safe must be a boolean")

redaction_status = str(redaction_payload.get("status", ""))
redaction_applied_count = redaction_payload.get("applied_count")
redaction_replay_safe = redaction_payload.get("replay_safe")
redaction_reason_codes = parse_reason_codes(redaction_payload, "redaction")

if redaction_status not in {"pass", "fail"}:
    fail("redaction.status must be pass or fail")
if not isinstance(redaction_applied_count, int):
    fail("redaction.applied_count must be an integer")
if not isinstance(redaction_replay_safe, bool):
    fail("redaction.replay_safe must be a boolean")

decision_reasons: list[str] = []

if retention_status != "pass":
    decision_reasons.append("retention_status_not_pass")
if redaction_status != "pass":
    decision_reasons.append("redaction_status_not_pass")
if not retention_replay_safe:
    decision_reasons.append("retention_replay_safe_false")
if not redaction_replay_safe:
    decision_reasons.append("redaction_replay_safe_false")
if lane == "contract" and ci_fast_gate != "PASS":
    decision_reasons.append("ci_fast_gate_failed")

if not retention_reason_codes:
    decision_reasons.append("retention_reason_codes_missing")
if not redaction_reason_codes:
    decision_reasons.append("redaction_reason_codes_missing")

final_decision = "GO" if not decision_reasons else "NO-GO"
if not decision_reasons:
    decision_reasons.append("all channel retention/redaction evidence invariants satisfied")

combined_reason_codes = sorted(set(retention_reason_codes + redaction_reason_codes))
evidence_key = f"channel_retention_redaction:{lane}:v1"
reason_key = f"channel_retention_redaction_reason:{final_decision}:v1"

payload = {
    "schema_version": "kamn.channel.retention-redaction-evidence.v1",
    "generated_at": generated_at,
    "lane": lane,
    "evidence_key": evidence_key,
    "reason_key": reason_key,
    "retention": {
        "status": retention_status,
        "total_candidates": retention_total_candidates,
        "replay_safe": retention_replay_safe,
        "reason_codes": retention_reason_codes,
    },
    "redaction": {
        "status": redaction_status,
        "applied_count": redaction_applied_count,
        "replay_safe": redaction_replay_safe,
        "reason_codes": redaction_reason_codes,
    },
    "combined_reason_codes": combined_reason_codes,
    "ci_fast_gate": ci_fast_gate,
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
printf 'schema_version=kamn.channel.retention-redaction-evidence.v1\n'
printf 'evidence_key=channel_retention_redaction:%s:v1\n' "$lane"
printf 'reason_key=channel_retention_redaction_reason:%s:v1\n' "$final_decision"
printf 'final_decision=%s\n' "$final_decision"
