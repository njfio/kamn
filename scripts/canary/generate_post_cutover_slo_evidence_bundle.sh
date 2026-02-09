#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/canary/generate_post_cutover_slo_evidence_bundle.sh \
    --output-file <path> \
    --window-minutes <int> \
    --p95-latency-ms <int> \
    --max-p95-latency-ms <int> \
    --error-rate-bps <int> \
    --max-error-rate-bps <int> \
    --delivery-success-bps <int> \
    --min-delivery-success-bps <int> \
    --snapshot-age-seconds <int> \
    --max-snapshot-age-seconds <int> \
    --evidence-complete <true|false> \
    --ci-fast-gate <PASS|FAIL>
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

require_int() {
  local field="$1"
  local value="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    fail "${field} must be an integer"
  fi
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
window_minutes=""
p95_latency_ms=""
max_p95_latency_ms=""
error_rate_bps=""
max_error_rate_bps=""
delivery_success_bps=""
min_delivery_success_bps=""
snapshot_age_seconds=""
max_snapshot_age_seconds=""
evidence_complete_raw=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --window-minutes)
      window_minutes="${2:-}"
      shift 2
      ;;
    --p95-latency-ms)
      p95_latency_ms="${2:-}"
      shift 2
      ;;
    --max-p95-latency-ms)
      max_p95_latency_ms="${2:-}"
      shift 2
      ;;
    --error-rate-bps)
      error_rate_bps="${2:-}"
      shift 2
      ;;
    --max-error-rate-bps)
      max_error_rate_bps="${2:-}"
      shift 2
      ;;
    --delivery-success-bps)
      delivery_success_bps="${2:-}"
      shift 2
      ;;
    --min-delivery-success-bps)
      min_delivery_success_bps="${2:-}"
      shift 2
      ;;
    --snapshot-age-seconds)
      snapshot_age_seconds="${2:-}"
      shift 2
      ;;
    --max-snapshot-age-seconds)
      max_snapshot_age_seconds="${2:-}"
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

if [[ -z "$output_file" || -z "$window_minutes" || -z "$p95_latency_ms" || -z "$max_p95_latency_ms" || -z "$error_rate_bps" || -z "$max_error_rate_bps" || -z "$delivery_success_bps" || -z "$min_delivery_success_bps" || -z "$snapshot_age_seconds" || -z "$max_snapshot_age_seconds" || -z "$evidence_complete_raw" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all generator arguments are required"
fi

for pair in \
  "window_minutes:$window_minutes" \
  "p95_latency_ms:$p95_latency_ms" \
  "max_p95_latency_ms:$max_p95_latency_ms" \
  "error_rate_bps:$error_rate_bps" \
  "max_error_rate_bps:$max_error_rate_bps" \
  "delivery_success_bps:$delivery_success_bps" \
  "min_delivery_success_bps:$min_delivery_success_bps" \
  "snapshot_age_seconds:$snapshot_age_seconds" \
  "max_snapshot_age_seconds:$max_snapshot_age_seconds"; do
  field="${pair%%:*}"
  value="${pair#*:}"
  require_int "$field" "$value"
done

case "$ci_fast_gate" in
  PASS|FAIL) ;;
  *)
    fail "--ci-fast-gate must be PASS or FAIL"
    ;;
esac

evidence_complete="$(to_bool "evidence_complete" "$evidence_complete_raw")"
mkdir -p "$(dirname "$output_file")"

python3 - "$output_file" "$window_minutes" "$p95_latency_ms" "$max_p95_latency_ms" "$error_rate_bps" "$max_error_rate_bps" "$delivery_success_bps" "$min_delivery_success_bps" "$snapshot_age_seconds" "$max_snapshot_age_seconds" "$evidence_complete" "$ci_fast_gate" <<'PY'
import json
import pathlib
import sys
from datetime import datetime, timezone

output_file = pathlib.Path(sys.argv[1])
window_minutes = int(sys.argv[2])
p95_latency_ms = int(sys.argv[3])
max_p95_latency_ms = int(sys.argv[4])
error_rate_bps = int(sys.argv[5])
max_error_rate_bps = int(sys.argv[6])
delivery_success_bps = int(sys.argv[7])
min_delivery_success_bps = int(sys.argv[8])
snapshot_age_seconds = int(sys.argv[9])
max_snapshot_age_seconds = int(sys.argv[10])
evidence_complete = sys.argv[11] == "true"
ci_fast_gate = sys.argv[12]

decision_reasons: list[str] = []
if p95_latency_ms > max_p95_latency_ms:
    decision_reasons.append("p95-latency-threshold-exceeded")
if error_rate_bps > max_error_rate_bps:
    decision_reasons.append("error-rate-threshold-exceeded")
if delivery_success_bps < min_delivery_success_bps:
    decision_reasons.append("delivery-success-threshold-breached")
if snapshot_age_seconds > max_snapshot_age_seconds:
    decision_reasons.append("stale-snapshot-evidence")
if not evidence_complete:
    decision_reasons.append("incomplete-slo-evidence")
if ci_fast_gate != "PASS":
    decision_reasons.append("ci-fast-gate-failed")

final_decision = "GO" if not decision_reasons else "NO-GO"

payload = {
    "schema_version": "kamn.launch-slo.evidence.v1",
    "generated_at": datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "window_minutes": window_minutes,
    "metrics": {
        "p95_latency_ms": p95_latency_ms,
        "max_p95_latency_ms": max_p95_latency_ms,
        "error_rate_bps": error_rate_bps,
        "max_error_rate_bps": max_error_rate_bps,
        "delivery_success_bps": delivery_success_bps,
        "min_delivery_success_bps": min_delivery_success_bps,
        "snapshot_age_seconds": snapshot_age_seconds,
        "max_snapshot_age_seconds": max_snapshot_age_seconds,
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
print(f"snapshot_age_seconds={snapshot_age_seconds}")
print(f"max_snapshot_age_seconds={max_snapshot_age_seconds}")
PY
