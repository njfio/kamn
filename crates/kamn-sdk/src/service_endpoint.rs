use super::{
    SdkError, REQUEST_TIMEOUT_SECONDS_DEFAULT, REQUEST_TIMEOUT_SECONDS_EMPTY_REASON,
    REQUEST_TIMEOUT_SECONDS_ENV, REQUEST_TIMEOUT_SECONDS_FIELD,
    REQUEST_TIMEOUT_SECONDS_INVALID_REASON, REQUEST_TIMEOUT_SECONDS_NON_POSITIVE_REASON,
    SERVICE_TLS_CA_FILE_EMPTY_BUNDLE, SERVICE_TLS_CA_FILE_EMPTY_REASON, SERVICE_TLS_CA_FILE_ENV,
    SERVICE_TLS_CA_FILE_FIELD, SERVICE_TLS_CA_FILE_PARSE_FAILED, SERVICE_TLS_CA_FILE_READ_FAILED,
    SERVICE_TLS_CA_FILE_UTF8_REASON, SERVICE_TLS_HANDSHAKE_FAILED, SERVICE_TLS_SERVER_NAME_INVALID,
};
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use std::fs;
use std::io::{Cursor, Read, Write};
use std::net::{IpAddr, TcpStream};
use std::sync::Arc;
use std::time::Duration;

fn configure_stream_timeouts(stream: &TcpStream, timeout: Duration) -> Result<(), SdkError> {
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|_| SdkError::TransportFailure("failed to configure service read timeout"))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|_| SdkError::TransportFailure("failed to configure service write timeout"))?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServiceScheme {
    Http,
    Https,
}

impl ServiceScheme {
    fn default_port(self) -> u16 {
        match self {
            Self::Http => 80,
            Self::Https => 443,
        }
    }
}

pub(super) enum ServiceStream {
    Tcp(TcpStream),
    Tls(Box<StreamOwned<ClientConnection, TcpStream>>),
}

impl Read for ServiceStream {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.read(buffer),
            Self::Tls(stream) => stream.read(buffer),
        }
    }
}

impl Write for ServiceStream {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.write(buffer),
            Self::Tls(stream) => stream.write(buffer),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Tcp(stream) => stream.flush(),
            Self::Tls(stream) => stream.flush(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ServiceEndpoint {
    scheme: ServiceScheme,
    pub(super) host: String,
    pub(super) port: u16,
    base_path: String,
}

impl ServiceEndpoint {
    pub(super) fn parse(endpoint: &str) -> Result<Self, SdkError> {
        let trimmed = endpoint.trim();
        if trimmed.is_empty() {
            return Err(SdkError::InvalidInput {
                field: "service.endpoint",
                reason: "must not be empty",
            });
        }

        let (scheme, suffix) = if let Some(suffix) = trimmed.strip_prefix("http://") {
            (ServiceScheme::Http, suffix)
        } else if let Some(suffix) = trimmed.strip_prefix("https://") {
            (ServiceScheme::Https, suffix)
        } else {
            return Err(SdkError::InvalidInput {
                field: "service.endpoint",
                reason: "must start with http:// or https://",
            });
        };

        let (authority, base_path) = match suffix.split_once('/') {
            Some((authority, path)) => (
                authority,
                format!("/{path}").trim_end_matches('/').to_owned(),
            ),
            None => (suffix, String::new()),
        };
        if authority.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "service.endpoint",
                reason: "host is required",
            });
        }

        let (host, port) =
            super::service_http_io::parse_host_port(authority, scheme.default_port())?;
        Ok(Self {
            scheme,
            host,
            port,
            base_path,
        })
    }

    pub(super) fn route_path(&self, route: &str) -> String {
        if self.base_path.is_empty() {
            return route.to_owned();
        }
        format!("{}{}", self.base_path, route)
    }

    fn connect_tcp_stream(&self) -> Result<TcpStream, SdkError> {
        let timeout = Duration::from_secs(resolve_request_timeout_seconds()?);
        let stream = TcpStream::connect((self.host.as_str(), self.port))
            .map_err(|_| SdkError::TransportFailure("failed to connect to service endpoint"))?;
        configure_stream_timeouts(&stream, timeout)?;
        Ok(stream)
    }

    pub(super) fn connect_stream(&self) -> Result<ServiceStream, SdkError> {
        let tcp_stream = self.connect_tcp_stream()?;
        if self.scheme == ServiceScheme::Http {
            return Ok(ServiceStream::Tcp(tcp_stream));
        }

        let tls_client_config = resolve_tls_client_config()?;
        let server_name = resolve_tls_server_name(self.host.as_str())?;
        let connection = ClientConnection::new(tls_client_config, server_name)
            .map_err(|_| SdkError::TransportFailure(SERVICE_TLS_HANDSHAKE_FAILED))?;
        Ok(ServiceStream::Tls(Box::new(StreamOwned::new(
            connection, tcp_stream,
        ))))
    }
}

fn resolve_tls_client_config() -> Result<Arc<ClientConfig>, SdkError> {
    let mut root_store = RootCertStore::empty();
    match std::env::var(SERVICE_TLS_CA_FILE_ENV) {
        Ok(configured_ca_file) => {
            let normalized_ca_file = configured_ca_file.trim();
            if normalized_ca_file.is_empty() {
                return Err(SdkError::InvalidInput {
                    field: SERVICE_TLS_CA_FILE_FIELD,
                    reason: SERVICE_TLS_CA_FILE_EMPTY_REASON,
                });
            }
            let cert_bytes = fs::read(normalized_ca_file)
                .map_err(|_| SdkError::TransportFailure(SERVICE_TLS_CA_FILE_READ_FAILED))?;
            let mut cert_reader = Cursor::new(cert_bytes.as_slice());
            let certificates = rustls_pemfile::certs(&mut cert_reader)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| SdkError::TransportFailure(SERVICE_TLS_CA_FILE_PARSE_FAILED))?;
            let (added, _) = root_store.add_parsable_certificates(certificates);
            if added == 0 {
                return Err(SdkError::TransportFailure(SERVICE_TLS_CA_FILE_EMPTY_BUNDLE));
            }
        }
        Err(std::env::VarError::NotPresent) => {
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        }
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err(SdkError::InvalidInput {
                field: SERVICE_TLS_CA_FILE_FIELD,
                reason: SERVICE_TLS_CA_FILE_UTF8_REASON,
            });
        }
    }

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    Ok(Arc::new(config))
}

pub(super) fn resolve_request_timeout_seconds() -> Result<u64, SdkError> {
    match std::env::var(REQUEST_TIMEOUT_SECONDS_ENV) {
        Ok(raw_timeout) => {
            let normalized_timeout = raw_timeout.trim();
            if normalized_timeout.is_empty() {
                return Err(SdkError::InvalidInput {
                    field: REQUEST_TIMEOUT_SECONDS_FIELD,
                    reason: REQUEST_TIMEOUT_SECONDS_EMPTY_REASON,
                });
            }
            let parsed_timeout =
                normalized_timeout
                    .parse::<u64>()
                    .map_err(|_| SdkError::InvalidInput {
                        field: REQUEST_TIMEOUT_SECONDS_FIELD,
                        reason: REQUEST_TIMEOUT_SECONDS_INVALID_REASON,
                    })?;
            if parsed_timeout == 0 {
                return Err(SdkError::InvalidInput {
                    field: REQUEST_TIMEOUT_SECONDS_FIELD,
                    reason: REQUEST_TIMEOUT_SECONDS_NON_POSITIVE_REASON,
                });
            }
            Ok(parsed_timeout)
        }
        Err(std::env::VarError::NotPresent) => Ok(REQUEST_TIMEOUT_SECONDS_DEFAULT),
        Err(std::env::VarError::NotUnicode(_)) => Err(SdkError::InvalidInput {
            field: REQUEST_TIMEOUT_SECONDS_FIELD,
            reason: REQUEST_TIMEOUT_SECONDS_INVALID_REASON,
        }),
    }
}

fn resolve_tls_server_name(host: &str) -> Result<ServerName<'static>, SdkError> {
    if let Ok(ip_addr) = host.parse::<IpAddr>() {
        return Ok(ServerName::IpAddress(ip_addr.into()));
    }
    ServerName::try_from(host.to_owned())
        .map_err(|_| SdkError::TransportFailure(SERVICE_TLS_SERVER_NAME_INVALID))
}
