# Plan: Issue 6210 - Bound Anti-Spam Seen-Message-ID Growth

- Issue: #6210
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Extend `AntiSpamConfig` with a deterministic capacity (`max_seen_message_ids`) and
   validate it at engine construction.
2. Replace unbounded seen-id retention with bounded FIFO retention:
   - keep membership checks in `HashSet<String>`
   - track insertion order in `VecDeque<String>`
   - evict oldest entries when capacity is exceeded
3. Add regressions for:
   - deterministic oldest-entry eviction behavior at capacity
   - invalid zero-capacity configuration rejection
4. Run scoped lint/tests for `kamn-runtime-guards` and dependent `kamn-node`.

## Affected Modules

- `crates/kamn-runtime-guards/src/anti_spam.rs`
- `crates/kamn-node/src/service_api_endpoint/server.rs` (compile compatibility with config defaults)

## Risks and Mitigations

1. Risk: eviction can allow very old message IDs to be replayed once evicted.
   - Mitigation: bounded memory is the explicit priority for this issue; future TTL/persistence work remains out of scope.
2. Risk: new config field could break struct literal call sites.
   - Mitigation: keep call sites using `..AntiSpamConfig::default()` and run dependent crate checks.
