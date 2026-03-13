use crate::SdkError;

#[path = "tcp/envelope.rs"]
mod envelope;
#[path = "tcp/handshake.rs"]
mod handshake;
#[path = "tcp/support.rs"]
mod support;
#[cfg(test)]
#[path = "tcp/tests.rs"]
mod tests;
#[path = "tcp/transport.rs"]
mod transport;

pub use envelope::{signature_for_fields, TcpSignedEnvelope};
pub use transport::{TcpReceivedEnvelope, TcpTransportAdapter, TcpTransportConfig};

#[allow(dead_code)]
fn _keep_sdk_error_used(_: &SdkError) {}
