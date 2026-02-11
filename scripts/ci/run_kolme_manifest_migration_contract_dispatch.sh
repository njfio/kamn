#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MANIFEST_RUNNER="$ROOT_DIR/scripts/framework/run_manifest_lane.sh"
DISPATCH_SCRIPT="$ROOT_DIR/scripts/ci/kolme_manifest_migration_contract.py"
DEFAULT_CONFIG="$ROOT_DIR/fixtures/ci/kolme_manifest_migration_contract_groups.json"

usage() {
  cat <<'EOF'
Usage: run_kolme_manifest_migration_contract_dispatch.sh --group <group-key> [--config-file <path>]
EOF
}

GROUP_KEY=""
CONFIG_FILE="$DEFAULT_CONFIG"

while (($# > 0)); do
  case "$1" in
    --group)
      if (($# < 2)); then
        echo "expected value after --group" >&2
        usage >&2
        exit 1
      fi
      GROUP_KEY="$2"
      shift 2
      ;;
    --config-file)
      if (($# < 2)); then
        echo "expected value after --config-file" >&2
        usage >&2
        exit 1
      fi
      CONFIG_FILE="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [ -z "$GROUP_KEY" ]; then
  echo "expected --group to be provided" >&2
  usage >&2
  exit 1
fi

if [ ! -f "$MANIFEST_RUNNER" ]; then
  echo "expected manifest wrapper runner to exist" >&2
  exit 1
fi

if [ ! -f "$DISPATCH_SCRIPT" ]; then
  echo "expected dispatcher script to exist: $DISPATCH_SCRIPT" >&2
  exit 1
fi

exec python3 "$DISPATCH_SCRIPT" \
  --root-dir "$ROOT_DIR" \
  --config-file "$CONFIG_FILE" \
  --group "$GROUP_KEY"
