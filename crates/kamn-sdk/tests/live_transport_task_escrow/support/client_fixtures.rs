use crate::support::did;
use kamn_sdk::{Artifact, EscrowConfig, LiveTransportKamnClient, TaskDefinition, TokenAmount};

pub(crate) fn deterministic_u64_tag(value: &str) -> u64 {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        acc ^= u64::from(*byte);
        acc = acc.wrapping_mul(0x00000100000001B3);
    }
    acc
}

pub(crate) fn live_client(endpoint: &str) -> LiveTransportKamnClient {
    let endpoint = format!("http://{endpoint}");
    LiveTransportKamnClient::connect(endpoint.as_str()).expect("live client should connect")
}

pub(crate) fn live_task() -> TaskDefinition {
    TaskDefinition {
        creator: did("creator-live"),
        task_type: "triage".to_owned(),
        description: "triage contract".to_owned(),
    }
}

pub(crate) fn live_escrow() -> EscrowConfig {
    EscrowConfig {
        payer: did("payer-live"),
        payee: did("payee-live"),
        amount: TokenAmount(7),
    }
}

pub(crate) fn live_artifact() -> Artifact {
    Artifact {
        name: "artifact.bin".to_owned(),
        bytes: b"artifact-bytes".to_vec(),
    }
}
