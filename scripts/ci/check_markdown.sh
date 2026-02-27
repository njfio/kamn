#!/usr/bin/env bash
set -euo pipefail

mapfile -t files < <(git ls-files '*.md')

if [ "${#files[@]}" -eq 0 ]; then
  echo "No markdown files tracked."
  exit 0
fi

status=0
for file in "${files[@]}"; do
  if grep -n '[[:blank:]]$' "$file" >/dev/null; then
    echo "Trailing whitespace found in $file"
    grep -n '[[:blank:]]$' "$file" || true
    status=1
  fi
done

if [ "$status" -ne 0 ]; then
  echo "Markdown whitespace checks failed."
  exit 1
fi

echo "Markdown whitespace checks passed."
