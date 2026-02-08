use kamn_sdk::{
    InMemoryKamnClient, KamnTransport, LiveTransportKamnClient, SdkError, TransportMode,
};

fn sanitize(value: &str) -> String {
    value.replace('\n', " ")
}

fn fail(reason: &str) -> ! {
    println!("status=error");
    println!("error={}", sanitize(reason));
    std::process::exit(1);
}

fn main() {
    let memory = InMemoryKamnClient::new();
    let live =
        match LiveTransportKamnClient::connect("https://live.kamn.testnet/profile-probe-rust") {
            Ok(value) => value,
            Err(error) => fail(&error.to_string()),
        };

    let (memory_expected, memory_found) = match memory.assert_transport_mode(TransportMode::Live) {
        Err(SdkError::TransportModeMismatch { expected, found }) => (expected, found),
        Err(error) => fail(&error.to_string()),
        Ok(()) => fail("memory client unexpectedly accepted live mode assertion"),
    };

    let (live_expected, live_found) = match live.assert_transport_mode(TransportMode::InMemory) {
        Err(SdkError::TransportModeMismatch { expected, found }) => (expected, found),
        Err(error) => fail(&error.to_string()),
        Ok(()) => fail("live client unexpectedly accepted in-memory mode assertion"),
    };

    println!("status=ok");
    println!(
        "default_transport_mode={}",
        memory.transport_mode().as_str()
    );
    println!("live_transport_mode={}", live.transport_mode().as_str());
    println!("memory_mismatch_expected={memory_expected}");
    println!("memory_mismatch_found={memory_found}");
    println!("live_mismatch_expected={live_expected}");
    println!("live_mismatch_found={live_found}");
}
