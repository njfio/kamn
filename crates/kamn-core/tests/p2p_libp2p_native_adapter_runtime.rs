#![cfg(feature = "libp2p-live-transport")]

#[path = "p2p_libp2p_native_adapter_runtime/config_contract_tests.rs"]
mod config_contract_tests;
#[path = "p2p_libp2p_native_adapter_runtime/discovery_gossip_contract_tests.rs"]
mod discovery_gossip_contract_tests;
#[path = "p2p_libp2p_native_adapter_runtime/partition_reason_contract_tests.rs"]
mod partition_reason_contract_tests;
#[path = "p2p_libp2p_native_adapter_runtime/performance_contract_tests.rs"]
mod performance_contract_tests;
#[path = "p2p_libp2p_native_adapter_runtime/runtime_marker_contract_tests.rs"]
mod runtime_marker_contract_tests;
#[path = "p2p_libp2p_native_adapter_runtime/support.rs"]
mod support;
