#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/deploy/preflight_topology.sh \
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

require_int() {
  local field="$1"
  local value="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    fail "${field} must be an integer"
  fi
}

processors=""
listeners=""
approvers=""
required_approvals=""

while [[ $# -gt 0 ]]; do
  case "$1" in
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

if [[ -z "$processors" || -z "$listeners" || -z "$approvers" || -z "$required_approvals" ]]; then
  usage
  fail "all topology arguments are required"
fi

require_int "processors" "$processors"
require_int "listeners" "$listeners"
require_int "approvers" "$approvers"
require_int "required-approvals" "$required_approvals"

if (( processors < 1 )); then
  fail "processors must be >= 1"
fi
if (( listeners < 1 )); then
  fail "listeners must be >= 1"
fi
if (( approvers < 1 )); then
  fail "approvers must be >= 1"
fi
if (( required_approvals < 1 || required_approvals > approvers )); then
  fail "required-approvals must be between 1 and approvers"
fi

printf 'status=ok\n'
printf 'processors=%s\n' "$processors"
printf 'listeners=%s\n' "$listeners"
printf 'approvers=%s\n' "$approvers"
printf 'required_approvals=%s\n' "$required_approvals"
