#!/usr/bin/env bash
set -euo pipefail

REGISTRY_FILE="${1:-.ci/flaky-tests.txt}"

if [ ! -f "$REGISTRY_FILE" ]; then
  echo "Flaky registry not found: $REGISTRY_FILE" >&2
  exit 1
fi

status=0
line_no=0

while IFS= read -r line || [ -n "$line" ]; do
  line_no=$(( line_no + 1 ))

  case "$line" in
    ""|\#*)
      continue
      ;;
  esac

  IFS='|' read -r owner test_id issue expiry notes extra <<<"$line"

  if [ -n "${extra:-}" ]; then
    echo "Invalid field count at $REGISTRY_FILE:$line_no" >&2
    status=1
    continue
  fi

  if [ -z "${owner:-}" ] || [ -z "${test_id:-}" ] || [ -z "${issue:-}" ] || [ -z "${expiry:-}" ] || [ -z "${notes:-}" ]; then
    echo "Missing required field at $REGISTRY_FILE:$line_no" >&2
    status=1
    continue
  fi

  if ! [[ "$issue" =~ ^#[0-9]+$ ]]; then
    echo "Invalid tracking issue format at $REGISTRY_FILE:$line_no (expected #<number>)" >&2
    status=1
  fi

  if ! date -u -d "$expiry" +%Y-%m-%d >/dev/null 2>&1; then
    echo "Invalid expiry date at $REGISTRY_FILE:$line_no (expected YYYY-MM-DD)" >&2
    status=1
    continue
  fi

  today="$(date -u +%Y-%m-%d)"
  if [ "$expiry" -lt "$today" ]; then
    echo "Expired flaky quarantine entry at $REGISTRY_FILE:$line_no (expiry: $expiry)" >&2
    status=1
  fi
done < "$REGISTRY_FILE"

if [ "$status" -ne 0 ]; then
  echo "Flaky registry validation failed." >&2
  exit 1
fi

echo "Flaky registry validation passed."
