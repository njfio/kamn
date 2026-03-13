use crate::SdkError;
use std::net::SocketAddr;

pub(super) fn parse_socket_addr(addr: &str) -> Result<SocketAddr, SdkError> {
    addr.parse().map_err(|_| SdkError::InvalidInput {
        field: "transport.addr",
        reason: "must be a valid host:port socket address",
    })
}

pub(super) fn validate_positive_u32(value: u32, field: &'static str) -> Result<(), SdkError> {
    validate_non_zero(value == 0, field)
}

pub(super) fn validate_positive_u64(value: u64, field: &'static str) -> Result<(), SdkError> {
    validate_non_zero(value == 0, field)
}

pub(super) fn validate_positive_usize(value: usize, field: &'static str) -> Result<(), SdkError> {
    validate_non_zero(value == 0, field)
}

fn validate_non_zero(is_zero: bool, field: &'static str) -> Result<(), SdkError> {
    if is_zero {
        return Err(SdkError::InvalidInput {
            field,
            reason: "must be greater than zero",
        });
    }
    Ok(())
}
