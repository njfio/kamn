#!/usr/bin/env bash
set -euo pipefail

MODE="fast"
DRY_RUN="false"

usage() {
  cat <<'EOF'
Usage: run_invariant_harness.sh [--mode fast|deep] [--dry-run]

Runs deterministic invariant harness tests with bounded seed sets.
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --mode)
      if [ "${2:-}" = "" ]; then
        echo "missing value for --mode" >&2
        exit 2
      fi
      MODE="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN="true"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

case "$MODE" in
  fast)
    SEEDS=(13)
    ;;
  deep)
    SEEDS=(13 97 401)
    ;;
  *)
    echo "unsupported mode: $MODE" >&2
    exit 2
    ;;
esac

COMMANDS=()
for seed in "${SEEDS[@]}"; do
  COMMANDS+=("KAMN_INVARIANT_SEED=$seed cargo test -p kamn-core --locked --all-features --test invariant_harness -- --nocapture")
done

for command in "${COMMANDS[@]}"; do
  echo "$command"
  if [ "$DRY_RUN" != "true" ]; then
    bash -lc "$command"
  fi
done
