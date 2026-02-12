use kamn_kolme::{
    KolmeRuntimeCommitProviderError, KolmeRuntimeCommitTransportErrorKind,
    KolmeTransportIoClassification,
};

#[test]
fn unit_runtime_transport_contracts_map_transport_io_classification_to_provider_error() {
    assert_eq!(
        KolmeRuntimeCommitProviderError::from(KolmeTransportIoClassification::Timeout),
        KolmeRuntimeCommitProviderError::Timeout
    );
    assert_eq!(
        KolmeRuntimeCommitProviderError::from(KolmeTransportIoClassification::Unavailable {
            reason: "transport io error: reset by peer".to_owned(),
        }),
        KolmeRuntimeCommitProviderError::Unavailable {
            reason: "transport io error: reset by peer".to_owned(),
        }
    );
}

#[test]
fn functional_runtime_transport_contracts_provider_error_display_is_deterministic() {
    assert_eq!(
        KolmeRuntimeCommitProviderError::Timeout.to_string(),
        "provider request timed out"
    );
    assert_eq!(
        KolmeRuntimeCommitProviderError::Unavailable {
            reason: "dial tcp refused".to_owned(),
        }
        .to_string(),
        "provider unavailable: dial tcp refused"
    );
    assert_eq!(
        KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "missing status field".to_owned(),
        }
        .to_string(),
        "provider malformed response: missing status field"
    );
}

#[test]
fn regression_issue_2277_runtime_transport_contract_error_kind_shape_stays_stable() {
    // Regression: #2277
    assert_eq!(
        format!("{:?}", KolmeRuntimeCommitTransportErrorKind::Timeout),
        "Timeout"
    );
    assert_eq!(
        format!("{:?}", KolmeRuntimeCommitTransportErrorKind::Unavailable),
        "Unavailable"
    );
    assert_eq!(
        format!(
            "{:?}",
            KolmeRuntimeCommitTransportErrorKind::MalformedResponse
        ),
        "MalformedResponse"
    );
}
