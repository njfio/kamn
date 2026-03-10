use k256::ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey};
use kamn_core::{
    KolmeApiBroadcastRequest, KolmeApiNextNonceRequest, KolmeCommitReceiptFinality,
    KolmeRuntimeCommitFinalityChecker, KolmeRuntimeCommitHttpTransport,
    KolmeRuntimeCommitBlockFallbackTransport,
    KolmeRuntimeCommitLiveProvider, KolmeRuntimeCommitProvider, KolmeRuntimeCommitProviderError,
    KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitRequest,
};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{env, fs};

const TLS_CA_FILE_ENV: &str = "KAMN_KOLME_TLS_CA_FILE";
const KOLME_LIVE_SIGNING_PROFILE_ENV: &str = "KAMN_KOLME_LIVE_SIGNING_PROFILE";
const KOLME_FORK_SECP256K1_PROFILE: &str = "kolme-fork-secp256k1-v1";
const KOLME_FORK_LIVE_SMOKE_SECRET_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

#[path = "kolme_runtime_commit_http_transport/support.rs"]
mod support;
#[path = "kolme_runtime_commit_http_transport/http_transport_contract_tests.rs"]
mod http_transport_contract_tests;
#[path = "kolme_runtime_commit_http_transport/typed_broadcast_contract_tests.rs"]
mod typed_broadcast_contract_tests;
#[path = "kolme_runtime_commit_http_transport/tls_transport_contract_tests.rs"]
mod tls_transport_contract_tests;
#[path = "kolme_runtime_commit_http_transport/fork_profile_contract_tests.rs"]
mod fork_profile_contract_tests;
#[path = "kolme_runtime_commit_http_transport/live_smoke_contract_tests.rs"]
mod live_smoke_contract_tests;

use support::*;
