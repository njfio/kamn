#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_version_compatibility_contract_lane.sh"
REPLAY_RUNNER="$ROOT_DIR/scripts/kolme/run_version_compatibility_replay.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/kolme_compatibility/version_compatibility_cases.json"

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
  output_json="$ROOT_DIR/kolme-version-compatibility-report.json"
fi

mkdir -p "$(dirname "$output_json")"

bash "$CONTRACT_LANE" >/dev/null

replay_output="$(
  python3 "$REPLAY_RUNNER" \
    --fixture "$FIXTURE_FILE" \
    --output-json "$output_json"
)"

if ! printf '%s\n' "$replay_output" | grep -q '^status=pass;'; then
  echo "expected Kolme version compatibility deep replay to pass" >&2
  exit 1
fi

echo "Kolme version compatibility replay deep lane tests passed."
