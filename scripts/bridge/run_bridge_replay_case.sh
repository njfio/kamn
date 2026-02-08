#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

usage() {
  echo "usage: run_bridge_replay_case.sh --suite <bridge_adapter|telegram_bridge|discord_bridge|cross_chain_bridge> --test-name <test>" >&2
  exit 2
}

suite=""
test_name=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --suite)
      shift
      [ "$#" -gt 0 ] || usage
      suite="$1"
      ;;
    --test-name)
      shift
      [ "$#" -gt 0 ] || usage
      test_name="$1"
      ;;
    *)
      usage
      ;;
  esac
  shift
done

[ -n "$suite" ] || usage
[ -n "$test_name" ] || usage

case "$suite" in
  bridge_adapter|telegram_bridge|discord_bridge|cross_chain_bridge)
    ;;
  *)
    echo "status=error"
    echo "suite=$suite"
    echo "test_name=$test_name"
    echo "error=unknown suite"
    exit 2
    ;;
esac

set +e
command_output="$(cargo test -p kamn-core --test "$suite" -- "$test_name" --exact 2>&1)"
command_code=$?
set -e

if [ "$command_code" -eq 0 ]; then
  if printf '%s\n' "$command_output" | grep -q "running 0 tests"; then
    echo "status=fail"
    echo "suite=$suite"
    echo "test_name=$test_name"
    echo "error=test-not-found"
    exit 1
  fi

  echo "status=pass"
  echo "suite=$suite"
  echo "test_name=$test_name"
  exit 0
fi

message="$(printf '%s' "$command_output" | tr '\n' ' ' | sed 's/[[:space:]]\+/ /g' | sed 's/^ //; s/ $//')"

echo "status=fail"
echo "suite=$suite"
echo "test_name=$test_name"
echo "error=$message"
exit 1
