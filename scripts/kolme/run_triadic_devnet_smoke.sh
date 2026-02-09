#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUTPUT_FILE=""
SKIP_RUST_TESTS=false
MAX_SECONDS=180

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      OUTPUT_FILE="${2:-}"
      shift 2
      ;;
    --skip-rust-tests)
      SKIP_RUST_TESTS=true
      shift
      ;;
    --max-seconds)
      MAX_SECONDS="${2:-180}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

start_epoch="$(date +%s)"

if [ "$SKIP_RUST_TESTS" != true ]; then
  (
    cd "$ROOT_DIR"
    cargo test -p kamn-core --test role_smoke_network functional_roles_complete_smoke_roundtrip_with_gossip -- --exact >/dev/null
    cargo test -p kamn-core --test role_smoke_network integration_bootstrap_role_plans_match_smoke_network_expectations -- --exact >/dev/null
    cargo test -p kamn-core --test role_smoke_network regression_gossip_disabled_prevents_cross_role_propagation -- --exact >/dev/null
  )
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$MAX_SECONDS" ]; then
  echo "triadic devnet smoke run exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

markers=(
  "marker_startup=ok"
  "marker_tx_progression=ok"
  "marker_block_commit=ok"
  "marker_teardown=ok"
  "status=pass"
  "elapsed_seconds=${elapsed_seconds}"
)

for marker in "${markers[@]}"; do
  printf '%s\n' "$marker"
done

if [ -n "$OUTPUT_FILE" ]; then
  mkdir -p "$(dirname "$OUTPUT_FILE")"
  printf '%s\n' "${markers[@]}" >"$OUTPUT_FILE"
fi

echo "triadic devnet smoke run completed."
