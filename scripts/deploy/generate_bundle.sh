#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/deploy/generate_bundle.sh \
    --output-file <path> \
    --processors <n> \
    --listeners <n> \
    --approvers <n> \
    --required-approvals <n>
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PREFLIGHT_SCRIPT="$ROOT_DIR/scripts/deploy/preflight_topology.sh"

output_file=""
processors=""
listeners=""
approvers=""
required_approvals=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --processors)
      processors="${2:-}"
      shift 2
      ;;
    --listeners)
      listeners="${2:-}"
      shift 2
      ;;
    --approvers)
      approvers="${2:-}"
      shift 2
      ;;
    --required-approvals)
      required_approvals="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if [[ -z "$output_file" || -z "$processors" || -z "$listeners" || -z "$approvers" || -z "$required_approvals" ]]; then
  usage
  fail "all generator arguments are required"
fi

mkdir -p "$(dirname "$output_file")"

# Reuse preflight checks so generation and validation cannot drift.
bash "$PREFLIGHT_SCRIPT" \
  --processors "$processors" \
  --listeners "$listeners" \
  --approvers "$approvers" \
  --required-approvals "$required_approvals" >/dev/null

cat >"$output_file" <<EOF
PROCESSORS=$processors
LISTENERS=$listeners
APPROVERS=$approvers
REQUIRED_APPROVALS=$required_approvals
EOF

printf 'status=generated\n'
printf 'bundle_file=%s\n' "$output_file"
