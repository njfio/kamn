use super::support::{constant_time_eq_bytes, split_transport_payload};

#[test]
fn unit_constant_time_compare_rejects_mismatch() {
    assert!(!constant_time_eq_bytes(b"abc", b"abd"));
}

#[test]
fn unit_split_transport_payload_requires_delimiter() {
    assert!(split_transport_payload("frame=handshake").is_err());
}
