#!/usr/bin/env bash
set -euo pipefail

REGISTRY_FILE="${1:-.ci/flaky-tests.txt}"

if [ ! -f "$REGISTRY_FILE" ]; then
  echo "Flaky registry not found: $REGISTRY_FILE" >&2
  exit 1
fi

total=0

echo "# Flaky Registry Report"
echo
echo "| Owner | Test ID | Tracking Issue | Expiry | Notes |"
echo "|---|---|---|---|---|"

while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in
    ""|\#*)
      continue
      ;;
  esac

  IFS='|' read -r owner test_id issue expiry notes _ <<<"$line"
  total=$(( total + 1 ))
  printf '| %s | %s | %s | %s | %s |\n' "$owner" "$test_id" "$issue" "$expiry" "$notes"
done < "$REGISTRY_FILE"

echo
echo "Total entries: $total"
