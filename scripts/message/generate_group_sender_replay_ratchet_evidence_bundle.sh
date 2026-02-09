#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/message/generate_group_sender_replay_ratchet_evidence_bundle.sh \
    --output-file <path> \
    --lane contract|deep \
    --report-file <path> \
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
report_file=""
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
    --report-file)
      report_file="${2:-}"
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

if [[ -z "$output_file" || -z "$lane" || -z "$report_file" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all group sender replay/ratchet evidence bundle arguments are required"
fi

if [[ ! -f "$report_file" ]]; then
  fail "report file not found: $report_file"
fi

mkdir -p "$(dirname "$output_file")"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$lane" "$report_file" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys


def fail(message: str) -> None:
    raise ValueError(message)


output_file, generated_at, lane, report_file, ci_fast_gate = sys.argv[1:]
if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")
if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci-fast-gate must be PASS or FAIL")

report = json.loads(pathlib.Path(report_file).read_text())
status = str(report.get("status", ""))
nonce_replay_detected = report.get("nonce_replay_detected")
stale_generation_detected = report.get("stale_generation_detected")
signature_tamper_detected = report.get("signature_tamper_detected")
reason_codes = report.get("reason_codes", [])

if status not in {"pass", "fail"}:
    fail("report.status must be pass or fail")
if not isinstance(nonce_replay_detected, bool):
    fail("report.nonce_replay_detected must be a boolean")
if not isinstance(stale_generation_detected, bool):
    fail("report.stale_generation_detected must be a boolean")
if not isinstance(signature_tamper_detected, bool):
    fail("report.signature_tamper_detected must be a boolean")
if not isinstance(reason_codes, list) or not all(
    isinstance(item, str) and item for item in reason_codes
):
    fail("report.reason_codes must be an array of non-empty strings")

reason_codes = sorted(reason_codes)

decision_reasons: list[str] = []
if status != "pass":
    decision_reasons.append("replay_ratchet_status_not_pass")
if nonce_replay_detected:
    decision_reasons.append("nonce_replay_detected")
if stale_generation_detected:
    decision_reasons.append("stale_generation_payload_detected")
if signature_tamper_detected:
    decision_reasons.append("signature_tamper_detected")
if lane == "contract" and ci_fast_gate != "PASS":
    decision_reasons.append("ci_fast_gate_failed")
if not reason_codes:
    decision_reasons.append("reason_codes_missing")

final_decision = "GO" if not decision_reasons else "NO-GO"
if not decision_reasons:
    decision_reasons.append("all group sender replay/ratchet checks passed")

evidence_key = f"group_sender_replay_ratchet:{lane}:v1"
reason_key = f"group_sender_replay_ratchet_reason:{final_decision}:v1"

payload = {
    "schema_version": "kamn.group-sender.replay-ratchet-evidence.v1",
    "generated_at": generated_at,
    "lane": lane,
    "evidence_key": evidence_key,
    "reason_key": reason_key,
    "report": {
        "status": status,
        "nonce_replay_detected": nonce_replay_detected,
        "stale_generation_detected": stale_generation_detected,
        "signature_tamper_detected": signature_tamper_detected,
        "reason_codes": reason_codes,
    },
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
printf 'schema_version=kamn.group-sender.replay-ratchet-evidence.v1\n'
printf 'evidence_key=group_sender_replay_ratchet:%s:v1\n' "$lane"
printf 'reason_key=group_sender_replay_ratchet_reason:%s:v1\n' "$final_decision"
printf 'final_decision=%s\n' "$final_decision"
