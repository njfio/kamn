#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
CI_TOOLS_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_tools.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -f "$FAST_WORKFLOW" ]; then
  echo "expected fast-gate workflow to exist" >&2
  exit 1
fi

if [ ! -f "$CI_TOOLS_SCRIPT" ]; then
  echo "expected aggregate CI tools script to exist" >&2
  exit 1
fi

mapfile -t kolme_tests < <(find "$ROOT_DIR/scripts/kolme" -maxdepth 1 -type f -name 'test_*.sh' | sort)
if [ "${#kolme_tests[@]}" -eq 0 ]; then
  echo "expected Kolme command-surface tests under scripts/kolme" >&2
  exit 1
fi

actual_fast_only=()
actual_ci_tools_only=()

for script_path in "${kolme_tests[@]}"; do
  relative_path="${script_path#"$ROOT_DIR/"}"
  in_fast=false
  in_ci_tools=false

  if grep -Fq "$relative_path" "$FAST_WORKFLOW"; then
    in_fast=true
  fi
  if grep -Fq "$relative_path" "$CI_TOOLS_SCRIPT"; then
    in_ci_tools=true
  fi

  if [ "$in_fast" = true ] && [ "$in_ci_tools" = false ]; then
    actual_fast_only+=("$relative_path")
  elif [ "$in_fast" = false ] && [ "$in_ci_tools" = true ]; then
    actual_ci_tools_only+=("$relative_path")
  fi
done

expected_fast_only=(
  "scripts/kolme/test_check_snapshot_drift.sh"
  "scripts/kolme/test_run_local_heavy_validation_matrix.sh"
  "scripts/kolme/test_run_local_runtime_commit_live_lane.sh"
  "scripts/kolme/test_run_snapshot_drift_contract_lane.sh"
  "scripts/kolme/test_run_triadic_devnet_smoke_contract_lane.sh"
  "scripts/kolme/test_run_version_compatibility_contract_lane.sh"
  "scripts/kolme/test_validate_version_compatibility.sh"
)

expected_ci_tools_only=(
  "scripts/kolme/test_check_nonce_broadcast_parity_policy.sh"
  "scripts/kolme/test_check_runtime_commit_replay_policy.sh"
  "scripts/kolme/test_run_block_fallback_reconciliation_contract_lane.sh"
  "scripts/kolme/test_run_nonce_broadcast_parity_contract_lane.sh"
  "scripts/kolme/test_run_notifications_consumer_contract_lane.sh"
  "scripts/kolme/test_run_runtime_commit_contract_lane.sh"
  "scripts/kolme/test_run_runtime_commit_replay_contract_lane.sh"
)

printf '%s\n' "${actual_fast_only[@]}" | sed '/^$/d' | sort >"$TMP_DIR/actual_fast_only.txt"
printf '%s\n' "${actual_ci_tools_only[@]}" | sed '/^$/d' | sort >"$TMP_DIR/actual_ci_tools_only.txt"
printf '%s\n' "${expected_fast_only[@]}" | sort >"$TMP_DIR/expected_fast_only.txt"
printf '%s\n' "${expected_ci_tools_only[@]}" | sort >"$TMP_DIR/expected_ci_tools_only.txt"

echo "kolme_fast_only_count_actual=$(wc -l <"$TMP_DIR/actual_fast_only.txt" | tr -d ' ')"
echo "kolme_fast_only_count_expected=$(wc -l <"$TMP_DIR/expected_fast_only.txt" | tr -d ' ')"
echo "kolme_ci_tools_only_count_actual=$(wc -l <"$TMP_DIR/actual_ci_tools_only.txt" | tr -d ' ')"
echo "kolme_ci_tools_only_count_expected=$(wc -l <"$TMP_DIR/expected_ci_tools_only.txt" | tr -d ' ')"

if ! diff -u "$TMP_DIR/expected_fast_only.txt" "$TMP_DIR/actual_fast_only.txt" >/dev/null; then
  echo "expected fast-only Kolme command-surface set to match approved policy" >&2
  diff -u "$TMP_DIR/expected_fast_only.txt" "$TMP_DIR/actual_fast_only.txt" >&2
  exit 1
fi

if ! diff -u "$TMP_DIR/expected_ci_tools_only.txt" "$TMP_DIR/actual_ci_tools_only.txt" >/dev/null; then
  echo "expected ci-tools-only Kolme command-surface set to match approved policy" >&2
  diff -u "$TMP_DIR/expected_ci_tools_only.txt" "$TMP_DIR/actual_ci_tools_only.txt" >&2
  exit 1
fi

echo "kolme command-surface asymmetry contract tests passed."
