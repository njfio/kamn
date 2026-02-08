#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/deploy/preflight_topology.sh \
    --bundle-file <path>

Or:

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
bundle_file=""

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
    --bundle-file)
      bundle_file="${2:-}"
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

trim_whitespace() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
}

if [[ -n "$bundle_file" ]]; then
  if [[ -n "$processors" || -n "$listeners" || -n "$approvers" || -n "$required_approvals" ]]; then
    fail "cannot mix --bundle-file with explicit topology arguments"
  fi

  if [[ ! -f "$bundle_file" ]]; then
    fail "bundle file not found: $bundle_file"
  fi

  while IFS='=' read -r raw_key raw_value; do
    key="$(trim_whitespace "${raw_key:-}")"
    value="$(trim_whitespace "${raw_value:-}")"

    if [[ -z "$key" || "${key:0:1}" == "#" ]]; then
      continue
    fi

    case "$key" in
      PROCESSORS)
        processors="$value"
        ;;
      LISTENERS)
        listeners="$value"
        ;;
      APPROVERS)
        approvers="$value"
        ;;
      REQUIRED_APPROVALS)
        required_approvals="$value"
        ;;
    esac
  done <"$bundle_file"

  if [[ -z "$processors" ]]; then
    fail "missing bundle field: PROCESSORS"
  fi
  if [[ -z "$listeners" ]]; then
    fail "missing bundle field: LISTENERS"
  fi
  if [[ -z "$approvers" ]]; then
    fail "missing bundle field: APPROVERS"
  fi
  if [[ -z "$required_approvals" ]]; then
    fail "missing bundle field: REQUIRED_APPROVALS"
  fi
elif [[ -z "$processors" || -z "$listeners" || -z "$approvers" || -z "$required_approvals" ]]; then
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
if [[ -n "$bundle_file" ]]; then
  printf 'bundle_file=%s\n' "$bundle_file"
fi
