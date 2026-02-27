# Plan: Issue #6041

## Approach
1. Introduce a process-global cache for `NodeLogConfig` resolution.
2. Keep parse/validation logic unchanged and reuse current `resolve_log_config_from_inputs`.
3. Add `#[cfg(test)]` cache reset helper for deterministic tests.
4. Add RED test for cache reset + env change visibility, then implement GREEN.

## Affected Modules
- `crates/kamn-node/src/logging.rs`

## Risks / Mitigations
- Risk: cache might hide env changes expected by existing tests.
  Mitigation: provide test-only reset helper and use it at test boundaries.
- Risk: cache initialization races.
  Mitigation: use `std::sync::OnceLock` + cloneable `NodeLogConfig`.

## Interfaces / Contracts
- No public API changes.
- Internal logging behavior: env resolution becomes cached after first successful load.
