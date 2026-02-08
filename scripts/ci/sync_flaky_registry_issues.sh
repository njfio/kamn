#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: $0 --repo <owner/repo> [--registry <path>] [--label <name>] [--dry-run]

Synchronizes flaky registry entries to GitHub issues by:
- adding a label to each tracking issue
- posting/updating an automated status comment

Registry format:
  owner|test-id|#issue|expires-on|notes
USAGE
}

REPO=""
REGISTRY=".ci/flaky-tests.txt"
LABEL="flaky-test"
DRY_RUN=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --repo)
      REPO="${2:-}"
      shift 2
      ;;
    --registry)
      REGISTRY="${2:-.ci/flaky-tests.txt}"
      shift 2
      ;;
    --label)
      LABEL="${2:-flaky-test}"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=true
      shift
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

if [ -z "$REPO" ]; then
  usage >&2
  exit 2
fi

if [ ! -f "$REGISTRY" ]; then
  echo "Registry file not found: $REGISTRY" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/check_flaky_registry.sh" "$REGISTRY" >/dev/null

if [ "$DRY_RUN" = false ] && ! command -v gh >/dev/null 2>&1; then
  echo "gh is required" >&2
  exit 2
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

# issue_num -> lines
# shellcheck disable=SC2034
declare -A ISSUE_LINES=()

total=0
while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in
    ""|\#*)
      continue
      ;;
  esac

  IFS='|' read -r owner test_id issue expiry notes _ <<<"$line"
  issue_num="${issue#\#}"

  entry="- ${owner} | ${test_id} | expires ${expiry} | ${notes}"
  if [ -n "${ISSUE_LINES[$issue_num]:-}" ]; then
    ISSUE_LINES[$issue_num]="${ISSUE_LINES[$issue_num]}\n${entry}"
  else
    ISSUE_LINES[$issue_num]="${entry}"
  fi
  total=$(( total + 1 ))
done < "$REGISTRY"

if [ "$total" -eq 0 ]; then
  echo "No flaky entries to sync."
  exit 0
fi

now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

for issue_num in "${!ISSUE_LINES[@]}"; do
  comment_body="$(cat <<BODY
Flaky Registry Sync (automated) at ${now}

The following flaky quarantine entries currently reference this issue:

${ISSUE_LINES[$issue_num]}

If an entry is no longer needed, remove it from \
".ci/flaky-tests.txt" and keep owner/expiry updated.
BODY
)"

  if [ "$DRY_RUN" = true ]; then
    echo "[dry-run] issue #${issue_num}"
    echo "[dry-run] add label: ${LABEL}"
    echo "[dry-run] comment body:"
    echo "$comment_body"
    echo
    continue
  fi

  gh issue view "$issue_num" -R "$REPO" >/dev/null
  gh issue edit "$issue_num" -R "$REPO" --add-label "$LABEL" >/dev/null

  # Keep one rolling automated comment instead of creating weekly duplicates.
  gh issue comment "$issue_num" -R "$REPO" --edit-last --create-if-none --body "$comment_body" >/dev/null
  echo "Synced flaky registry to issue #${issue_num}"
done

echo "Flaky registry sync complete for ${#ISSUE_LINES[@]} issue(s)."
