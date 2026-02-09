#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/compliance/run_soc2_control_evidence_contract_lane.sh"
MATRIX_SCRIPT="$ROOT_DIR/scripts/compliance/run_soc2_control_evidence_replay_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/compliance_soc2/control_evidence_replay_cases.json"

output_json=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -z "$output_json" ]]; then
  output_json="$ROOT_DIR/soc2-control-evidence-report.json"
fi

mkdir -p "$(dirname "$output_json")"

bash "$CONTRACT_LANE" >/dev/null

matrix_output="$(
  python3 "$MATRIX_SCRIPT" \
    --fixture "$FIXTURE_FILE" \
    --output-json "$output_json"
)"

if ! printf '%s\n' "$matrix_output" | grep -q '^status=pass;'; then
  echo "expected SOC2 control evidence deep matrix to pass" >&2
  exit 1
fi

echo "soc2 control evidence deep lane tests passed."

