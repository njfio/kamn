#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SMOKE_RUNNER="$ROOT_DIR/scripts/kolme/run_triadic_devnet_smoke.sh"
VALIDATOR="$ROOT_DIR/scripts/kolme/validate_triadic_devnet_smoke.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/kolme_compatibility/devnet_smoke_markers.json"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
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
  output_json="$ROOT_DIR/triadic-devnet-smoke-report.json"
fi

if [ ! -x "$SMOKE_RUNNER" ]; then
  echo "expected triadic devnet smoke runner to be executable" >&2
  exit 1
fi

if [ ! -x "$VALIDATOR" ]; then
  echo "expected triadic devnet smoke validator to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected triadic devnet smoke marker fixture file to exist" >&2
  exit 1
fi

if [ ! -f "$DOC_FILE" ] || [ ! -f "$README_FILE" ]; then
  echo "expected triadic devnet docs to exist" >&2
  exit 1
fi

mkdir -p "$(dirname "$output_json")"
tmp_markers="$(mktemp)"
trap 'rm -f "$tmp_markers"' EXIT

start_epoch="$(date +%s)"

bash "$SMOKE_RUNNER" --output-file "$tmp_markers" --max-seconds 180 >/dev/null
python3 "$VALIDATOR" \
  --fixture "$FIXTURE_FILE" \
  --marker-file "$tmp_markers" \
  --output-json "$output_json" >/dev/null

if ! grep -q "run_triadic_devnet_smoke.sh" "$DOC_FILE"; then
  echo "expected devnet ops doc to reference triadic devnet smoke runner command" >&2
  exit 1
fi

if ! grep -q "validate_triadic_devnet_smoke.py" "$DOC_FILE"; then
  echo "expected devnet ops doc to reference triadic devnet smoke validator command" >&2
  exit 1
fi

if ! grep -q "run_triadic_devnet_smoke.sh" "$README_FILE"; then
  echo "expected README to reference triadic devnet smoke command" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 180 ]; then
  echo "triadic devnet smoke contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "triadic devnet smoke contract lane tests passed."
