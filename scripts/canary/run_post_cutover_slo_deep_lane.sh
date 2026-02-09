#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/canary/generate_post_cutover_slo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/canary/check_post_cutover_slo_policy.sh"
CONTRACT_LANE="$ROOT_DIR/scripts/canary/run_post_cutover_slo_contract_lane.sh"

start_epoch="$(date +%s)"
max_runtime="${KAMN_POST_CUTOVER_SLO_DEEP_MAX_SECONDS:-180}"
if [[ ! "$max_runtime" =~ ^[0-9]+$ ]]; then
  echo "KAMN_POST_CUTOVER_SLO_DEEP_MAX_SECONDS must be an integer >= 0" >&2
  exit 1
fi

report_file="$ROOT_DIR/post-cutover-slo-report.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      report_file="${2:-}"
      shift 2
      ;;
    --help|-h)
      cat <<'EOF'
Usage:
  bash scripts/canary/run_post_cutover_slo_deep_lane.sh \
    [--output-json <path>]
EOF
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

mkdir -p "$(dirname "$report_file")"

bash "$CONTRACT_LANE"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$report_file" \
    --window-minutes 15 \
    --p95-latency-ms 245 \
    --max-p95-latency-ms 200 \
    --error-rate-bps 18 \
    --max-error-rate-bps 25 \
    --delivery-success-bps 9992 \
    --min-delivery-success-bps 9950 \
    --snapshot-age-seconds 360 \
    --max-snapshot-age-seconds 120 \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected post-cutover SLO deep-lane stale scenario decision to be NO-GO" >&2
  exit 1
fi
if ! printf '%s\n' "$generator_output" | grep -q "^reason_key=slo_alert_reason_codes:NO-GO:v1$"; then
  echo "expected post-cutover SLO deep-lane stale scenario reason_key to be NO-GO schema marker" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$report_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected post-cutover SLO deep-lane policy decision to be NO-GO" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^reason_key=slo_alert_reason_codes:NO-GO:v1$"; then
  echo "expected post-cutover SLO deep-lane policy reason_key to be NO-GO schema marker" >&2
  exit 1
fi

runtime_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$runtime_seconds" -gt "$max_runtime" ]; then
  echo "post-cutover SLO deep lane exceeded runtime budget (${runtime_seconds}s > ${max_runtime}s)" >&2
  exit 1
fi

echo "post-cutover SLO deep lane tests passed."
