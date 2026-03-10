use super::{
    default_endpoint, kolme_endpoint, DEFAULT_S08_AGENT_NAME, DEFAULT_S08_POST_MESSAGE_PAYLOAD,
    DEFAULT_S08_PRE_MESSAGE_PAYLOAD, DEFAULT_S09_AGENT_NAME, DEFAULT_S09_POST_MESSAGE_PAYLOAD,
    DEFAULT_S09_PRE_MESSAGE_PAYLOAD,
};

#[path = "recovery_failover_probes/crash_recovery_probe.rs"]
mod crash_recovery_probe;
#[path = "recovery_failover_probes/transport_failover_probe.rs"]
mod transport_failover_probe;

pub(super) fn run_live_s08_crash_recovery_probe() -> Result<(), String> {
    crash_recovery_probe::run_live_s08_crash_recovery_probe()
}

pub(super) fn run_live_s09_transport_failover_probe() -> Result<(), String> {
    transport_failover_probe::run_live_s09_transport_failover_probe()
}

pub(super) struct S08Settings {
    pub(super) endpoint: String,
    pub(super) kolme_endpoint: String,
    pub(super) base_agent_name: String,
    pub(super) pre_message_payload: String,
    pub(super) post_message_payload: String,
}

pub(super) struct S09Settings {
    pub(super) primary_endpoint: String,
    pub(super) failover_endpoint: String,
    pub(super) kolme_endpoint: String,
    pub(super) base_agent_name: String,
    pub(super) pre_message_payload: String,
    pub(super) post_message_payload: String,
}

pub(super) fn s08_settings() -> S08Settings {
    S08Settings {
        endpoint: default_endpoint(),
        kolme_endpoint: kolme_endpoint(),
        base_agent_name: super::super::env_var_or_default(
            "KAMN_E2E_S08_AGENT_NAME",
            DEFAULT_S08_AGENT_NAME,
        ),
        pre_message_payload: super::super::env_var_or_default(
            "KAMN_E2E_S08_PRE_MESSAGE_PAYLOAD",
            DEFAULT_S08_PRE_MESSAGE_PAYLOAD,
        ),
        post_message_payload: super::super::env_var_or_default(
            "KAMN_E2E_S08_POST_MESSAGE_PAYLOAD",
            DEFAULT_S08_POST_MESSAGE_PAYLOAD,
        ),
    }
}

pub(super) fn s09_settings() -> S09Settings {
    let primary_endpoint = default_endpoint();
    S09Settings {
        failover_endpoint: super::super::env_var_or_else("KAMN_E2E_S09_FAILOVER_ENDPOINT", || {
            primary_endpoint.clone()
        }),
        kolme_endpoint: kolme_endpoint(),
        base_agent_name: super::super::env_var_or_default(
            "KAMN_E2E_S09_AGENT_NAME",
            DEFAULT_S09_AGENT_NAME,
        ),
        pre_message_payload: super::super::env_var_or_default(
            "KAMN_E2E_S09_PRE_MESSAGE_PAYLOAD",
            DEFAULT_S09_PRE_MESSAGE_PAYLOAD,
        ),
        post_message_payload: super::super::env_var_or_default(
            "KAMN_E2E_S09_POST_MESSAGE_PAYLOAD",
            DEFAULT_S09_POST_MESSAGE_PAYLOAD,
        ),
        primary_endpoint,
    }
}
