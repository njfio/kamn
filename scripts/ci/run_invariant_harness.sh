#!/usr/bin/env bash
set -euo pipefail

MODE="fast"
DRY_RUN="false"
PARALLELISM=1

usage() {
  cat <<'EOF'
Usage: run_invariant_harness.sh [--mode fast|deep] [--parallelism <n>] [--dry-run]

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
    --parallelism)
      if [ "${2:-}" = "" ]; then
        echo "missing value for --parallelism" >&2
        exit 2
      fi
      PARALLELISM="$2"
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

if ! [[ "$PARALLELISM" =~ ^[0-9]+$ ]] || [ "$PARALLELISM" -lt 1 ] || [ "$PARALLELISM" -gt 4 ]; then
  echo "--parallelism must be an integer between 1 and 4" >&2
  exit 2
fi

COMMANDS=()
for seed in "${SEEDS[@]}"; do
  COMMANDS+=("KAMN_INVARIANT_SEED=$seed cargo test -p kamn-core --locked --all-features --test invariant_harness -- --nocapture")
done

if [ "$DRY_RUN" = "true" ] || [ "$PARALLELISM" -eq 1 ] || [ "${#COMMANDS[@]}" -le 1 ]; then
  for command in "${COMMANDS[@]}"; do
    echo "$command"
    if [ "$DRY_RUN" != "true" ]; then
      bash -lc "$command"
    fi
  done
  exit 0
fi

failures=0
running=0
for command in "${COMMANDS[@]}"; do
  echo "$command"
  bash -lc "$command" &
  running=$(( running + 1 ))

  if [ "$running" -ge "$PARALLELISM" ]; then
    if ! wait -n; then
      failures=1
    fi
    running=$(( running - 1 ))
  fi
done

while [ "$running" -gt 0 ]; do
  if ! wait -n; then
    failures=1
  fi
  running=$(( running - 1 ))
done

if [ "$failures" -ne 0 ]; then
  exit 1
fi
