#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/bridge/generate_localhost_bridge_demo_evidence_bundle.sh \
    --output-file <path> \
    --lane contract|deep \
    --relay-lane-output-file <path> \
    --replay-report-file <path> \
    --ci-fast-gate PASS|FAIL
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

output_file=""
lane=""
relay_lane_output_file=""
replay_report_file=""
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
    --relay-lane-output-file)
      relay_lane_output_file="${2:-}"
      shift 2
      ;;
    --replay-report-file)
      replay_report_file="${2:-}"
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

if [[ -z "$output_file" || -z "$lane" || -z "$relay_lane_output_file" || -z "$replay_report_file" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all localhost bridge demo evidence bundle arguments are required"
fi

if [[ ! -f "$relay_lane_output_file" ]]; then
  fail "relay lane output file not found: $relay_lane_output_file"
fi

if [[ ! -f "$replay_report_file" ]]; then
  fail "replay report file not found: $replay_report_file"
fi

mkdir -p "$(dirname "$output_file")"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$lane" "$relay_lane_output_file" "$replay_report_file" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys


def fail(message: str) -> None:
    raise ValueError(message)


def read_marker(output: str, key: str) -> str:
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return "missing"


(
    output_file,
    generated_at,
    lane,
    relay_lane_output_file,
    replay_report_file,
    ci_fast_gate,
) = sys.argv[1:]

if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")
if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci-fast-gate must be PASS or FAIL")

relay_output = pathlib.Path(relay_lane_output_file).read_text()
replay_payload = json.loads(pathlib.Path(replay_report_file).read_text())

signed_transport = read_marker(relay_output, "bridge_demo_signed_transport")
relay_contracts = read_marker(relay_output, "bridge_demo_relay_contracts")
completion_marker_present = "localhost bridge relay demo contract lane tests passed." in relay_output

replay_status = str(replay_payload.get("status", ""))
case_count = int(replay_payload.get("case_count", 0))
failed_count = int(replay_payload.get("failed_count", 0))
requested_suites = replay_payload.get("requested_suites", [])
failed_case_ids = replay_payload.get("failed_case_ids", [])

if not isinstance(requested_suites, list):
    fail("replay report requested_suites must be an array")
if not isinstance(failed_case_ids, list):
    fail("replay report failed_case_ids must be an array")

decision_reasons: list[str] = []

if signed_transport != "pass":
    decision_reasons.append("localhost signed transport marker is not pass")
if relay_contracts != "pass":
    decision_reasons.append("localhost bridge relay contracts marker is not pass")
if not completion_marker_present:
    decision_reasons.append("localhost bridge relay completion marker is missing")
if replay_status != "pass":
    decision_reasons.append("bridge replay matrix status is not pass")
if case_count <= 0:
    decision_reasons.append("bridge replay matrix must include at least one case")
if failed_count > 0:
    decision_reasons.append("bridge replay matrix reported failed cases")
if lane == "contract" and ci_fast_gate != "PASS":
    decision_reasons.append("ci-fast-gate-failed")

final_decision = "GO" if not decision_reasons else "NO-GO"
if not decision_reasons:
    decision_reasons.append("all localhost bridge demo evidence invariants satisfied")

payload = {
    "schema_version": "kamn.bridge.localhost-demo-evidence.v1",
    "generated_at": generated_at,
    "lane": lane,
    "relay": {
        "signed_transport": signed_transport,
        "relay_contracts": relay_contracts,
        "completion_marker_present": completion_marker_present,
    },
    "replay": {
        "status": replay_status,
        "case_count": case_count,
        "failed_count": failed_count,
        "requested_suites": requested_suites,
        "failed_case_ids": failed_case_ids,
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
