#[path = "live_transport_http.rs"]
mod http;

use std::net::TcpListener;
use std::thread;
use std::time::Duration;

pub(crate) use self::http::parse_http_request;

pub(crate) fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

pub(crate) fn wait_for_server_ready() {
    thread::sleep(Duration::from_millis(40));
}
