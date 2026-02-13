#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
STRATEGY_CONTRACT_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_strategy_contract.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
WAVE_RANGE_MARKER_PATTERN='non_kolme_wrapper_family_wave_range=[0-9]+-[0-9]+'

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
tmp_doc="$tmp_dir/strategy.md"
tmp_output="$tmp_dir/strategy-contract-output.log"

cp "$STRATEGY_DOC" "$tmp_doc"

wave_marker="$(grep -Eo "$WAVE_RANGE_MARKER_PATTERN" "$tmp_doc" | head -n 1 || true)"
if [ -z "$wave_marker" ]; then
  echo "expected strategy document fixture to contain non_kolme_wrapper_family_wave_range marker" >&2
  exit 1
fi

wave_range="${wave_marker#*=}"
wave_start="${wave_range%-*}"
wave_end="${wave_range##*-}"
if ! [[ "$wave_start" =~ ^[0-9]+$ && "$wave_end" =~ ^[0-9]+$ ]]; then
  echo "expected non_kolme_wrapper_family_wave_range marker to include numeric bounds" >&2
  exit 1
fi

mutated_wave_end=$((wave_end + 1))
mutated_wave_marker="non_kolme_wrapper_family_wave_range=${wave_start}-${mutated_wave_end}"
sed -i "0,/${wave_marker}/s//${mutated_wave_marker}/" "$tmp_doc"

if KAMN_CI_STRATEGY_DOC_FILE="$tmp_doc" bash "$STRATEGY_CONTRACT_SCRIPT" >"$tmp_output" 2>&1; then
  echo "expected strategy contract to fail when non_kolme_wrapper_family_wave_range diverges from documented snippets" >&2
  exit 1
fi

expected_missing_snippet="CI strategy contract failed: missing snippet 'non_kolme_wave${mutated_wave_end}_wrapper_family_matrix.json'."
if ! grep -Fq "$expected_missing_snippet" "$tmp_output"; then
  echo "expected deterministic missing-snippet failure when non_kolme_wrapper_family_wave_range is widened" >&2
  cat "$tmp_output" >&2
  exit 1
fi

echo "ci strategy wave-range marker contract regression tests passed."
