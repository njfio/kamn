#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: $0 --repo <owner/repo> --issue <number> [--registry <path>] [--dry-run]

Posts flaky registry report as an issue comment.
Requires: gh, bash scripts/ci/report_flaky_registry.sh
USAGE
}

REPO=""
ISSUE=""
REGISTRY=".ci/flaky-tests.txt"
DRY_RUN=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --repo)
      REPO="${2:-}"
      shift 2
      ;;
    --issue)
      ISSUE="${2:-}"
      shift 2
      ;;
    --registry)
      REGISTRY="${2:-.ci/flaky-tests.txt}"
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

if [ -z "$REPO" ] || [ -z "$ISSUE" ]; then
  usage >&2
  exit 2
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "gh is required" >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

report_file="$TMP_DIR/flaky-report.md"
header_file="$TMP_DIR/comment.md"

bash "$SCRIPT_DIR/report_flaky_registry.sh" "$REGISTRY" > "$report_file"

{
  echo "Automated flaky registry report ($(date -u +%Y-%m-%dT%H:%M:%SZ))."
  echo
  cat "$report_file"
} > "$header_file"

if [ "$DRY_RUN" = true ]; then
  cat "$header_file"
  exit 0
fi

gh issue comment "$ISSUE" -R "$REPO" --body-file "$header_file"
