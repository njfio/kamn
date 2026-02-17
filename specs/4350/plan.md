# Plan — #4350

Status: Reviewed

## Approach

- Modify `scripts/ci/check_kamn_core_missing_docs_policy.sh` to:
  - parse throughput report JSON fields;
  - parse velocity policy JSON fields;
  - print stable `missing_docs_*` evidence markers.
- On velocity guard failure, forward deterministic policy markers before exiting.

## Risks

- Parsing errors from malformed JSON.
  - Mitigation: bounded helper extraction via `python3` with strict key handling.
