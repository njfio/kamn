use super::*;

pub(crate) fn fork_provider(base_url: &str, provider_hint: &str) -> KolmeRuntimeCommitLiveProvider<KolmeRuntimeCommitHttpTransport> {
    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(base_url, provider_hint, transport)
        .expect("provider should build")
}

pub(crate) fn fork_txhash_server(txhash: &str, matcher: impl Fn(String) + Send + 'static) -> String {
    spawn_single_request_server(
        format!("{{\"txhash\":\"{txhash}\"}}"),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("PUT /broadcast HTTP/1.1"));
            matcher(request);
        },
    )
}

pub(crate) fn assert_pending_fork_receipt(outcome: KolmeRuntimeCommitProviderOutcome, provider_hint: &str, txhash: &str) {
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, provider_hint);
            assert_eq!(receipt.commit_id, format!("kolme-commit:{txhash}"));
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Pending);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}
