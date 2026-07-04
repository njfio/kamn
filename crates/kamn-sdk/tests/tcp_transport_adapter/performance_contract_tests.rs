use super::support::*;

#[test]
fn performance_tcp_adapter_local_relay_contract_stays_within_budget() {
    let addr = free_addr();
    let (listener_adapter, sender_adapter) = adapter_pair(addr.as_str());
    let envelope = build_envelope(
        did("sender-perf"),
        did("listener-perf"),
        1,
        "state:perf",
        "perf-envelope",
    );
    let started = Instant::now();
    let listener_thread = listen_once_in_thread(listener_adapter);
    wait_for_listener();
    sender_adapter
        .send(&envelope)
        .unwrap_or_else(|error| panic!("sender adapter failed: {error}"));
    let received =
        join_listener(listener_thread).unwrap_or_else(|error| panic!("listener failed: {error}"));
    assert_eq!(received.envelope, envelope);
    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 500,
        "tcp adapter relay contract lane exceeded budget: {elapsed_millis}ms"
    );
}
