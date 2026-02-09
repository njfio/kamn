#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/bridge/generate_bridge_replay_redaction_evidence_bundle.sh \
    --output-file <path> \
    --lane contract|deep \
    --replay-report-file <path> \
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
replay_report_file=""
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
    --replay-report-file)
      replay_report_file="${2:-}"
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

if [[ -z "$output_file" || -z "$lane" || -z "$replay_report_file" || -z "$redaction_report_file" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all bridge replay/redaction evidence bundle arguments are required"
fi

if [[ ! -f "$replay_report_file" ]]; then
  fail "replay report file not found: $replay_report_file"
fi

if [[ ! -f "$redaction_report_file" ]]; then
  fail "redaction report file not found: $redaction_report_file"
fi

mkdir -p "$(dirname "$output_file")"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$lane" "$replay_report_file" "$redaction_report_file" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys


def fail(message: str) -> None:
    raise ValueError(message)


(
    output_file,
    generated_at,
    lane,
    replay_report_file,
    redaction_report_file,
    ci_fast_gate,
) = sys.argv[1:]

if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")
if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci-fast-gate must be PASS or FAIL")

replay_payload = json.loads(pathlib.Path(replay_report_file).read_text())
redaction_payload = json.loads(pathlib.Path(redaction_report_file).read_text())

replay_status = str(replay_payload.get("status", ""))
case_count = int(replay_payload.get("case_count", 0))
failed_count = int(replay_payload.get("failed_count", 0))
requested_suites = replay_payload.get("requested_suites", [])
if not isinstance(requested_suites, list):
    fail("replay report requested_suites must be an array")
failed_case_ids = replay_payload.get("failed_case_ids", [])
if not isinstance(failed_case_ids, list):
    fail("replay report failed_case_ids must be an array")

redaction_status = str(redaction_payload.get("status", ""))
redaction_mode = str(redaction_payload.get("mode", ""))
connectors = redaction_payload.get("connectors", [])
if not isinstance(connectors, list):
    fail("redaction report connectors must be an array")
connector_count = len(connectors)
leaked_connectors = redaction_payload.get("leaked_connectors", [])
if leaked_connectors is None:
    leaked_connectors = []
if not isinstance(leaked_connectors, list):
    fail("redaction report leaked_connectors must be an array")
leaked_connector_count = len(leaked_connectors)

decision_reasons: list[str] = []

if replay_status != "pass":
    decision_reasons.append("bridge replay matrix status is not pass")
if case_count <= 0:
    decision_reasons.append("bridge replay matrix must include at least one case")
if failed_count > 0:
    decision_reasons.append("bridge replay matrix reported failed cases")
if redaction_status != "pass":
    decision_reasons.append("bridge redaction checker status is not pass")
if connector_count <= 0:
    decision_reasons.append("bridge redaction report must include connector samples")
if leaked_connector_count > 0:
    decision_reasons.append("bridge redaction report indicates leaked connectors")

expected_redaction_mode = "contract" if lane == "contract" else "deep"
if redaction_mode != expected_redaction_mode:
    decision_reasons.append(
        f"redaction mode mismatch: expected {expected_redaction_mode}, got {redaction_mode}"
    )

if lane == "contract" and ci_fast_gate != "PASS":
    decision_reasons.append("ci-fast-gate-failed")

final_decision = "GO" if not decision_reasons else "NO-GO"
if not decision_reasons:
    decision_reasons.append("all bridge replay/redaction evidence invariants satisfied")

payload = {
    "schema_version": "kamn.bridge.replay-redaction-evidence.v1",
    "generated_at": generated_at,
    "lane": lane,
    "replay": {
        "status": replay_status,
        "case_count": case_count,
        "failed_count": failed_count,
        "requested_suites": requested_suites,
        "failed_case_ids": failed_case_ids,
    },
    "redaction": {
        "status": redaction_status,
        "mode": redaction_mode,
        "connector_count": connector_count,
        "leaked_connector_count": leaked_connector_count,
        "leaked_connectors": leaked_connectors,
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
printf 'final_decision=%s\n' "$final_decision"
