use super::*;

#[test]
fn unit_live_provider_rejects_empty_endpoint_or_submit_path() {
    let (transport, _calls) = RecordingTransport::with_result(Ok(String::new()));
    assert!(
        matches!(
            KolmeRuntimeCommitLiveProvider::new("", "/broadcast/runtime-commit", transport),
            Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_base_url",
                reason: "must not be empty",
            })
        ),
        "provider base URL should fail validation when empty"
    );

    let (transport, _calls) = RecordingTransport::with_result(Ok(String::new()));
    assert!(
        matches!(
            KolmeRuntimeCommitLiveProvider::new("http://127.0.0.1:3030", "", transport),
            Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_submit_path",
                reason: "must not be empty",
            })
        ),
        "provider submit path should fail validation when empty"
    );
}

#[test]
fn unit_adapter_backed_client_rejects_empty_expected_provider() {
    let (provider, _calls) =
        RecordingProvider::with_result(Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "unused".to_owned(),
        }));
    assert!(
        matches!(
            AdapterBackedKolmeRuntimeCommitClient::new(" ", provider),
            Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "expected_provider",
                reason: "must not be empty",
            })
        ),
        "expected provider should fail validation when empty"
    );
}

#[test]
fn unit_in_memory_client_rejects_empty_provider() {
    assert!(
        matches!(
            InMemoryKolmeRuntimeCommitClient::new(""),
            Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider",
                reason: "must not be empty",
            })
        ),
        "in-memory provider should fail validation when empty"
    );
}
