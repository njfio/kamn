#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: $0 --label <name> [--max-attempts <n>] -- <command> [args...]
USAGE
}

write_output() {
  local key="$1"
  local value="$2"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    {
      echo "${key}<<EOF"
      echo "$value"
      echo "EOF"
    } >>"$GITHUB_OUTPUT"
  fi
}

append_summary() {
  if [ -z "${GITHUB_STEP_SUMMARY:-}" ]; then
    return
  fi

  {
    echo "### Retry Report (${LABEL})"
    echo "- Attempts used: ${ATTEMPT}"
    echo "- Max attempts: ${MAX_ATTEMPTS}"
    echo "- Retry used: ${RETRY_USED}"
    echo "- Final status: ${FINAL_STATUS}"
  } >>"$GITHUB_STEP_SUMMARY"
}

LABEL="command"
MAX_ATTEMPTS=2

while [ "$#" -gt 0 ]; do
  case "$1" in
    --label)
      LABEL="${2:-command}"
      shift 2
      ;;
    --max-attempts)
      MAX_ATTEMPTS="${2:-2}"
      shift 2
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

if [ "$#" -eq 0 ]; then
  usage >&2
  exit 2
fi

if ! [[ "$MAX_ATTEMPTS" =~ ^[0-9]+$ ]] || [ "$MAX_ATTEMPTS" -lt 1 ] || [ "$MAX_ATTEMPTS" -gt 3 ]; then
  echo "--max-attempts must be an integer between 1 and 3" >&2
  exit 2
fi

ATTEMPT=1
RETRY_USED=false
FINAL_STATUS=failed

until "$@"; do
  if [ "$ATTEMPT" -ge "$MAX_ATTEMPTS" ]; then
    FINAL_STATUS=failed
    write_output "retry_attempts" "$ATTEMPT"
    write_output "retry_used" "$RETRY_USED"
    write_output "retry_final_status" "$FINAL_STATUS"
    append_summary
    echo "${LABEL} failed after ${ATTEMPT} attempt(s)." >&2
    exit 1
  fi

  RETRY_USED=true
  ATTEMPT=$(( ATTEMPT + 1 ))
  echo "${LABEL} failed; retrying (attempt ${ATTEMPT}/${MAX_ATTEMPTS})..." >&2
  sleep 2
done

FINAL_STATUS=passed
write_output "retry_attempts" "$ATTEMPT"
write_output "retry_used" "$RETRY_USED"
write_output "retry_final_status" "$FINAL_STATUS"
append_summary

echo "${LABEL} succeeded after ${ATTEMPT} attempt(s)."
