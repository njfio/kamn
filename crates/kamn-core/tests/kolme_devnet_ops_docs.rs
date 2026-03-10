const PLAN: &str = include_str!("../../../docs/planning/kolme-devnet-ops.md");
const DEPLOY_COMPAT: &str = include_str!("../../../docs/deploy/kolme_devnet_ops.md");

#[path = "kolme_devnet_ops_docs/shared_support.rs"]
mod shared_support;
#[path = "kolme_devnet_ops_docs/service_api_failover_contract_tests.rs"]
mod service_api_failover_contract_tests;
#[path = "kolme_devnet_ops_docs/deploy_compat_contract_tests.rs"]
mod deploy_compat_contract_tests;
#[path = "kolme_devnet_ops_docs/local_lane_contract_tests.rs"]
mod local_lane_contract_tests;
#[path = "kolme_devnet_ops_docs/migration_manifest_contract_tests.rs"]
mod migration_manifest_contract_tests;
#[path = "kolme_devnet_ops_docs/regression_migration_contract_tests.rs"]
mod regression_migration_contract_tests;
#[path = "kolme_devnet_ops_docs/regression_local_lane_contract_tests.rs"]
mod regression_local_lane_contract_tests;
#[path = "kolme_devnet_ops_docs/runtime_transport_contract_tests.rs"]
mod runtime_transport_contract_tests;
