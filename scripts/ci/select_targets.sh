#!/usr/bin/env bash
set -euo pipefail

write_output() {
  local key="$1"
  local value="$2"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
      echo "${key}<<EOF"
      echo "$value"
      echo "EOF"
    } >>"$GITHUB_OUTPUT"
  else
    printf '%s=%s\n' "$key" "$value"
  fi
}

append_summary() {
  if [ -z "${GITHUB_STEP_SUMMARY:-}" ]; then
    return
  fi

  {
    echo "### CI Scope Selection"
    echo "- Base reference: ${BASE_REF_DISPLAY}"
    echo "- Changed files: ${CHANGED_COUNT}"
    echo "- Docs only: ${DOCS_ONLY}"
    echo "- Run Rust checks: ${RUN_RUST}"
    echo "- Run CI tool checks: ${RUN_CI_TOOL_CHECKS}"
    echo "- Run deploy preflight checks: ${RUN_DEPLOY_PREFLIGHT_TESTS}"
    echo "- Run bridge replay harness: ${RUN_BRIDGE_REPLAY_HARNESS}"
    echo "- Run SDK parity matrix: ${RUN_SDK_PARITY_MATRIX}"
    echo "- Run invariant harness: ${RUN_INVARIANT_HARNESS}"
    echo "- Test scope: ${TEST_SCOPE}"
    echo "- Critical path fallback: ${CRITICAL_PATH_CHANGED}"
    echo "- Unknown path fallback: ${UNKNOWN_RISK_CHANGED}"
    if [ -n "$CHANGED_MANIFESTS" ]; then
      echo "- Targeted manifests: ${CHANGED_MANIFESTS}"
    fi
  } >>"$GITHUB_STEP_SUMMARY"
}

find_manifest_for_path() {
  local path="$1"
  local dir

  dir="$(dirname "$path")"

  while [ "$dir" != "." ] && [ "$dir" != "/" ]; do
    if [ -f "$dir/Cargo.toml" ]; then
      printf '%s/Cargo.toml\n' "$dir"
      return 0
    fi
    local parent
    parent="$(dirname "$dir")"
    if [ "$parent" = "$dir" ]; then
      break
    fi
    dir="$parent"
  done

  if [ -f Cargo.toml ]; then
    printf 'Cargo.toml\n'
    return 0
  fi

  return 1
}

join_with_and() {
  local joined=""
  local part
  for part in "$@"; do
    if [ -z "$joined" ]; then
      joined="$part"
    else
      joined+=" && $part"
    fi
  done
  printf '%s\n' "$joined"
}

if git ls-files | grep -Eq '(^|/)Cargo.toml$'; then
  REPO_HAS_RUST=true
else
  REPO_HAS_RUST=false
fi

BASE_REF="${GITHUB_BASE_REF:-main}"
BASE_REF_DISPLAY="$BASE_REF"

if git rev-parse --verify "origin/${BASE_REF}" >/dev/null 2>&1; then
  BASE_COMMIT="$(git merge-base HEAD "origin/${BASE_REF}")"
elif git rev-parse --verify HEAD~1 >/dev/null 2>&1; then
  BASE_COMMIT="$(git rev-parse HEAD~1)"
else
  BASE_COMMIT=""
fi

if [ -n "${BASE_COMMIT}" ]; then
  BASE_REF_DISPLAY="$BASE_COMMIT"
  mapfile -t CHANGED_FILES < <(git diff --name-only "${BASE_COMMIT}...HEAD" | sed '/^$/d')
else
  BASE_REF_DISPLAY="(initial-snapshot)"
  mapfile -t CHANGED_FILES < <(git ls-files | sed '/^$/d')
fi

if [ -n "${CI_CHANGED_FILES:-}" ]; then
  BASE_REF_DISPLAY="(env:CI_CHANGED_FILES)"
  mapfile -t CHANGED_FILES < <(printf '%s\n' "$CI_CHANGED_FILES" | sed '/^$/d')
fi

CHANGED_COUNT="${#CHANGED_FILES[@]}"
DOCS_ONLY=true
RUST_CHANGED=false
CI_INFRA_CHANGED=false
DEPLOY_SCRIPT_CHANGED=false
BRIDGE_REPLAY_RELATED_CHANGED=false
SDK_RELATED_CHANGED=false
CRITICAL_PATH_CHANGED=false
UNKNOWN_RISK_CHANGED=false
FULL_SUITE=false
INVARIANT_RELATED_CHANGED=false

# manifest -> 1
# shellcheck disable=SC2034
declare -A MANIFESTS=()

for file in "${CHANGED_FILES[@]}"; do
  classified=false

  case "$file" in
    *.md|*.txt|docs/*)
      classified=true
      ;;
    *)
      DOCS_ONLY=false
      ;;
  esac

  case "$file" in
    *.rs|Cargo.toml|Cargo.lock|*/Cargo.toml|*/Cargo.lock|rust-toolchain.toml|.cargo/*)
      RUST_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    .github/workflows/*|scripts/ci/*)
      CI_INFRA_CHANGED=true
      CRITICAL_PATH_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    scripts/deploy/*)
      DEPLOY_SCRIPT_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    crates/kamn-core/src/bridge_adapter.rs|crates/kamn-core/src/telegram_bridge.rs|crates/kamn-core/src/discord_bridge.rs|crates/kamn-core/src/cross_chain_bridge.rs|crates/kamn-core/tests/bridge_adapter.rs|crates/kamn-core/tests/telegram_bridge.rs|crates/kamn-core/tests/discord_bridge.rs|crates/kamn-core/tests/cross_chain_bridge.rs|docs/foundation/bridge-adapter-abstraction.md|scripts/bridge/*|fixtures/bridge_replay/*)
      BRIDGE_REPLAY_RELATED_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    kamn_sdk.py|packages/kamn-sdk/*|crates/kamn-sdk/*|scripts/sdk/*|fixtures/sdk_parity/*|tests/python/test_sdk.py)
      SDK_RELATED_CHANGED=true
      classified=true
      ;;
  esac

  case "$file" in
    crates/kamn-core/src/invariants.rs|crates/kamn-core/src/transaction.rs|crates/kamn-core/src/smoke.rs|crates/kamn-core/tests/invariant_*|crates/kamn-core/tests/transaction_guards.rs|docs/foundation/invariants.md|docs/foundation/transaction-guards.md|scripts/ci/run_invariant_harness.sh|scripts/ci/test_run_invariant_harness.sh)
      INVARIANT_RELATED_CHANGED=true
      ;;
  esac

  case "$file" in
    Cargo.toml|Cargo.lock|rust-toolchain.toml|.cargo/*)
      FULL_SUITE=true
      ;;
  esac

  if [ "$DOCS_ONLY" = false ] && [ "$classified" = false ]; then
    UNKNOWN_RISK_CHANGED=true
  fi

  if manifest="$(find_manifest_for_path "$file" 2>/dev/null)"; then
    MANIFESTS["$manifest"]=1
  fi
done

if [ "$CRITICAL_PATH_CHANGED" = true ] || [ "$UNKNOWN_RISK_CHANGED" = true ]; then
  FULL_SUITE=true
fi

RUN_RUST=false
RUN_CI_TOOL_CHECKS=false
RUN_DEPLOY_PREFLIGHT_TESTS=false
RUN_BRIDGE_REPLAY_HARNESS=false
RUN_SDK_PARITY_MATRIX=false
FMT_CMD=":"
CLIPPY_CMD=":"
TEST_CMD=":"
TEST_SCOPE="none"
CHANGED_MANIFESTS=""
RUN_INVARIANT_HARNESS=false

if [ "$REPO_HAS_RUST" = true ] && { [ "$RUST_CHANGED" = true ] || [ "$CI_INFRA_CHANGED" = true ] || [ "$FULL_SUITE" = true ]; }; then
  RUN_RUST=true

  mapfile -t manifest_list < <(printf '%s\n' "${!MANIFESTS[@]}" | sed '/^$/d' | sort)

  if [ "${#manifest_list[@]}" -gt 0 ]; then
    CHANGED_MANIFESTS="$(printf '%s, ' "${manifest_list[@]}" | sed 's/, $//')"
  fi

  FMT_CMD='cargo fmt --all --check'

  if [ "$FULL_SUITE" = true ]; then
    TEST_SCOPE="full"
    CLIPPY_CMD='cargo clippy --workspace --all-targets --all-features -- -D warnings'
    TEST_CMD='bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test --workspace --locked --all-features --no-fail-fast'
  elif [ "${#manifest_list[@]}" -gt 1 ] || { [ "${#manifest_list[@]}" -eq 1 ] && [ "${manifest_list[0]}" != "Cargo.toml" ]; }; then
    TEST_SCOPE="targeted"

    clippy_parts=()
    test_parts=()
    for manifest in "${manifest_list[@]}"; do
      clippy_parts+=("cargo clippy --all-targets --all-features --manifest-path '$manifest' -- -D warnings")
      test_parts+=("bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test --locked --all-features --manifest-path '$manifest' --no-fail-fast")
    done

    CLIPPY_CMD="$(join_with_and "${clippy_parts[@]}")"
    TEST_CMD="$(join_with_and "${test_parts[@]}")"
  else
    TEST_SCOPE="smoke"
    CLIPPY_CMD='cargo clippy --workspace --all-targets -- -D warnings'
    TEST_CMD='bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test --workspace --locked --all-features --lib --no-fail-fast'
  fi
fi

if [ "$RUN_RUST" = true ] && { [ "$INVARIANT_RELATED_CHANGED" = true ] || [ "$FULL_SUITE" = true ]; }; then
  RUN_INVARIANT_HARNESS=true
fi

if [ "$RUN_RUST" != true ] && [ "$DEPLOY_SCRIPT_CHANGED" = true ]; then
  RUN_DEPLOY_PREFLIGHT_TESTS=true
  TEST_SCOPE="deploy"
fi

if [ "$BRIDGE_REPLAY_RELATED_CHANGED" = true ]; then
  RUN_BRIDGE_REPLAY_HARNESS=true
  if [ "$RUN_RUST" != true ] && [ "$TEST_SCOPE" = "none" ]; then
    TEST_SCOPE="bridge"
  fi
fi

if [ "$SDK_RELATED_CHANGED" = true ]; then
  RUN_SDK_PARITY_MATRIX=true
  if [ "$RUN_RUST" != true ]; then
    TEST_SCOPE="sdk"
  fi
fi

if [ "$CI_INFRA_CHANGED" = true ]; then
  RUN_CI_TOOL_CHECKS=true
fi

write_output "docs_only" "$DOCS_ONLY"
write_output "run_rust" "$RUN_RUST"
write_output "run_ci_tool_checks" "$RUN_CI_TOOL_CHECKS"
write_output "run_deploy_preflight_tests" "$RUN_DEPLOY_PREFLIGHT_TESTS"
write_output "run_bridge_replay_harness" "$RUN_BRIDGE_REPLAY_HARNESS"
write_output "run_sdk_parity_matrix" "$RUN_SDK_PARITY_MATRIX"
write_output "run_invariant_harness" "$RUN_INVARIANT_HARNESS"
write_output "test_scope" "$TEST_SCOPE"
write_output "critical_path_changed" "$CRITICAL_PATH_CHANGED"
write_output "unknown_risk_changed" "$UNKNOWN_RISK_CHANGED"
write_output "changed_files" "$CHANGED_COUNT"
write_output "changed_manifests" "$CHANGED_MANIFESTS"
write_output "fmt_cmd" "$FMT_CMD"
write_output "clippy_cmd" "$CLIPPY_CMD"
write_output "test_cmd" "$TEST_CMD"

append_summary
