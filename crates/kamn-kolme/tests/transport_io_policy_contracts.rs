use kamn_kolme::{
    classify_transport_io_error, KolmeTransportIoClassification as TransportIoClassification,
};
use std::io;

#[test]
fn unit_classify_transport_io_error_maps_timeout_and_would_block_to_timeout() {
    let timeout = io::Error::new(io::ErrorKind::TimedOut, "timed out");
    assert_eq!(
        classify_transport_io_error(&timeout),
        TransportIoClassification::Timeout
    );

    let would_block = io::Error::new(io::ErrorKind::WouldBlock, "would block");
    assert_eq!(
        classify_transport_io_error(&would_block),
        TransportIoClassification::Timeout
    );
}

#[test]
fn unit_classify_transport_io_error_maps_other_kinds_to_unavailable() {
    let reset = io::Error::new(io::ErrorKind::ConnectionReset, "reset by peer");
    assert_eq!(
        classify_transport_io_error(&reset),
        TransportIoClassification::Unavailable {
            reason: "transport io error: reset by peer".to_owned(),
        }
    );
}
