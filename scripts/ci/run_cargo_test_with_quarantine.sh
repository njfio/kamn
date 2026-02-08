#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: run_cargo_test_with_quarantine.sh [--registry <path>] [--dry-run] -- cargo test ...

Reads the flaky quarantine registry and appends --skip flags for active flaky tests.
USAGE
}

REGISTRY=".ci/flaky-tests.txt"
DRY_RUN=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --registry)
      REGISTRY="${2:-.ci/flaky-tests.txt}"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --)
      shift
      break
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [ "$#" -lt 2 ]; then
  usage >&2
  exit 2
fi

if [ "$1" != "cargo" ] || [ "${2:-}" != "test" ]; then
  echo "run_cargo_test_with_quarantine.sh requires a cargo test command" >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/check_flaky_registry.sh" "$REGISTRY" >/dev/null

declare -a SKIP_TESTS=()
while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in
    ""|\#*)
      continue
      ;;
  esac
  IFS='|' read -r _owner test_id _issue _expiry _notes _extra <<<"$line"
  SKIP_TESTS+=("$test_id")
done < "$REGISTRY"

declare -a COMMAND=("$@")

has_double_dash=false
for arg in "${COMMAND[@]}"; do
  if [ "$arg" = "--" ]; then
    has_double_dash=true
    break
  fi
done

if [ "${#SKIP_TESTS[@]}" -gt 0 ] && [ "$has_double_dash" = false ]; then
  COMMAND+=(--)
fi

if [ "${#SKIP_TESTS[@]}" -gt 0 ]; then
  for test_id in "${SKIP_TESTS[@]}"; do
    COMMAND+=(--skip "$test_id")
  done
fi

if [ "$DRY_RUN" = true ]; then
  printf '%q ' "${COMMAND[@]}"
  echo
  exit 0
fi

"${COMMAND[@]}"
