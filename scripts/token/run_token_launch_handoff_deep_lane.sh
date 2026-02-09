#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/token/generate_token_launch_handoff_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/token/check_token_launch_handoff_policy.sh"
CONTRACT_LANE="$ROOT_DIR/scripts/token/run_token_launch_handoff_contract_lane.sh"

report_file="$ROOT_DIR/token-launch-handoff-report.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      report_file="${2:-}"
      shift 2
      ;;
    --help|-h)
      cat <<'EOF'
Usage:
  bash scripts/token/run_token_launch_handoff_deep_lane.sh \
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
    --token-symbol "KAMN" \
    --configured-total-supply 1000000000 \
    --expected-total-supply 1000000000 \
    --configured-allocation-sum 999999000 \
    --expected-allocation-sum 1000000000 \
    --allocation-bucket-count 5 \
    --expected-bucket-count 5 \
    --genesis-hash "sha256:token-launch-handoff-deep-2026-02-09" \
    --required-approvals 3 \
    --received-approvals 2 \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected token launch handoff deep-lane mismatch scenario decision to be NO-GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$report_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected token launch handoff deep-lane policy decision to be NO-GO" >&2
  exit 1
fi

echo "token launch handoff deep lane tests passed."
