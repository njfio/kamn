#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

usage() {
  cat <<'EOF' >&2
usage: run_live_transport_parity_contract_lane.sh [--languages <rust,python,typescript>]
EOF
  exit 2
}

normalize_languages() {
  local raw="$1"
  local token
  local normalized=()
  local seen_rust=false
  local seen_python=false
  local seen_typescript=false

  if [ -z "$raw" ] || [ "$raw" = "all" ]; then
    printf 'rust,python,typescript\n'
    return 0
  fi

  IFS=',' read -r -a tokens <<<"$raw"
  for token in "${tokens[@]}"; do
    case "$(printf '%s' "$token" | tr '[:upper:]' '[:lower:]' | xargs)" in
      rust)
        if [ "$seen_rust" = false ]; then
          normalized+=("rust")
          seen_rust=true
        fi
        ;;
      python)
        if [ "$seen_python" = false ]; then
          normalized+=("python")
          seen_python=true
        fi
        ;;
      typescript)
        if [ "$seen_typescript" = false ]; then
          normalized+=("typescript")
          seen_typescript=true
        fi
        ;;
      "")
        ;;
      *)
        echo "unsupported language selector: $token" >&2
        usage
        ;;
    esac
  done

  if [ "${#normalized[@]}" -eq 0 ]; then
    echo "at least one language must be selected" >&2
    usage
  fi

  printf '%s,' "${normalized[@]}" | sed 's/,$//'
}

LANGUAGES="all"
while [ "$#" -gt 0 ]; do
  case "$1" in
    --languages)
      if [ "$#" -lt 2 ]; then
        usage
      fi
      LANGUAGES="$2"
      shift 2
      ;;
    *)
      usage
      ;;
  esac
done

SELECTED_LANGUAGES="$(normalize_languages "$LANGUAGES")"

RUN_RUST=false
RUN_PYTHON=false
RUN_TYPESCRIPT=false
for language in ${SELECTED_LANGUAGES//,/ }; do
  case "$language" in
    rust)
      RUN_RUST=true
      ;;
    python)
      RUN_PYTHON=true
      ;;
    typescript)
      RUN_TYPESCRIPT=true
      ;;
  esac
done

if [ "$RUN_RUST" = true ]; then
  echo "running rust live transport contract lane tests"
  bash "$ROOT_DIR/scripts/sdk/run_rust_live_transport_contract_lane.sh"
fi

if [ "$RUN_PYTHON" = true ]; then
  echo "running python live transport contract lane tests"
  python3 -m unittest tests/python/test_sdk.py
fi

if [ "$RUN_TYPESCRIPT" = true ]; then
  echo "running typescript live transport contract lane tests"
  npm --prefix packages/kamn-sdk test
fi

echo "running transport profile parity drift matrix checks"
bash "$ROOT_DIR/scripts/sdk/run_transport_profile_parity_matrix.sh" --languages "$SELECTED_LANGUAGES"

echo "live transport parity contract lane tests passed for languages: ${SELECTED_LANGUAGES}."
